#include <errno.h>
#include <limits.h>
#include <linux/capability.h>
#include <pwd.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/prctl.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

// provided by libcap-ng
#include <cap-ng.h>

#define BUF_SIZE 4096

static char buf[BUF_SIZE + 1];
static bool zsh_mode;

// forward declarations
static int dump_last_line();
static bool is_empty_file(const char *name);

#define MAX(X, Y) (((X) < (Y)) ? (Y) : (X))

int main(int argc, char *argv[]) {
  if (argc != 2) {
    exit(EXIT_FAILURE);
  }

  char *endptr;
  char *str = argv[1];
  errno = 0;

  unsigned long pane_pid = strtoul(str, &endptr, 10);
  if (errno) {
    perror("strtol");
    return EXIT_FAILURE;
  }
  if (endptr == str) {
    fprintf(stderr, "Unable to parse as number: %s\n", str);
    return EXIT_FAILURE;
  }

  snprintf(buf, sizeof(buf), "/proc/%lu/task/%lu/children", pane_pid, pane_pid);
  if (is_empty_file(buf)) {
    fprintf(stderr, "skipping pid %lu for it has no children\n", pane_pid);
    return EXIT_SUCCESS;
  }

  { // determine shell: bash or zsh
    snprintf(buf, sizeof(buf), "/proc/%lu/task/%lu/exe", pane_pid, pane_pid);
    char tmp[1024];
    ssize_t len;
    if ((len = readlink(buf, tmp, sizeof(tmp)-1)) != -1) {
      tmp[len] = '\0';
      if (strncmp("zsh", &tmp[MAX(0, len-3)], 3) == 0) {
        zsh_mode = true;
      }
    }
    fprintf(stderr, "detected shell: %s\n",  zsh_mode ? "zsh" : "bash");
  }

  if (capng_get_caps_process() == 0) {
    /* try to inherit CAP_SYS_PTRACE */
    int rc = capng_update(CAPNG_ADD, CAPNG_INHERITABLE, CAP_SYS_PTRACE);
    if (rc) {
      printf("Cannot add inheritable cap\n");
      exit(EXIT_FAILURE);
    }
    capng_apply(CAPNG_SELECT_CAPS);

    /* Note the two 0s at the end. Kernel checks for these */
    if (prctl(PR_CAP_AMBIENT, PR_CAP_AMBIENT_RAISE, CAP_SYS_PTRACE, 0, 0)) {
      perror("Cannot set ambient cap");
      exit(EXIT_FAILURE);
    }
  }

  pid_t child_pid = fork();
  if (child_pid < 0) {
    perror("fork");
    return EXIT_FAILURE;
  }

  if (child_pid == 0) {
    /* child */

    int count_attach = snprintf(buf, sizeof(buf), "attach %lu", pane_pid);
    if (count_attach < 0 || count_attach >= sizeof(buf)) {
      exit(EXIT_FAILURE);
    }

    char *argv[] = {
        "gdb", "-n",     "-batch", "--eval", &buf[0], "--eval",
        NULL,  "--eval", "detach", "--eval", "q",     NULL,
    };
    argv[6] = zsh_mode
                  ? "call (void)savehistfile(\"/tmp/gdb_history.txt\", 0, 0)"
                  : "(void)write_history(\"/tmp/gdb_history.txt\")";

    close(STDOUT_FILENO);
    // bye bye
    execvp("gdb", argv);
    perror("execv");
    exit(EXIT_FAILURE);
  } else {
    /* main process */
    int status;
    do {
      pid_t wpid = waitpid(child_pid, &status, WUNTRACED);
      if (wpid == -1) {
        perror("waitpid");
        exit(EXIT_FAILURE);
      }

      if (WIFEXITED(status)) {
        fprintf(stderr, "child exited, status=%d\n", WEXITSTATUS(status));
      } else if (WIFSIGNALED(status)) {
        fprintf(stderr, "child killed (signal %d)\n", WTERMSIG(status));
      } else { /* Non-standard case -- may never happen */
        fprintf(stderr, "Unexpected status (0x%x)\n", status);
      }
    } while (!WIFEXITED(status) && !WIFSIGNALED(status));

    int ret = dump_last_line();
    unlink("/tmp/gdb_history.txt");
    exit(ret);
  }

  return 0;
}

static int dump_last_line() {

  FILE *fd = fopen("/tmp/gdb_history.txt", "rb");
  if (fd == NULL) {
    perror("error opening history file");
    return EXIT_FAILURE;
  }

  int ret = EXIT_FAILURE;

  (void)fseek(fd, -sizeof(buf), SEEK_END);

  size_t bytes_read = fread(buf, 1, sizeof(buf) - 1, fd);
  if (bytes_read == 0) {
    fprintf(stderr, "fread() failed: %zu\n", bytes_read);
    goto finish;
  }
  buf[bytes_read] = '\0';

  const char *cmd = buf;

  // try to find newline
  char *last_newline = strrchr(buf, '\n');
  if (last_newline) {
    // go one past the newline char
    cmd = last_newline + 1;
  }

  if (zsh_mode) {
    // parse zsh history syntax
    cmd = strchr(cmd, ';');
    if (cmd) {
      ++cmd;
    }
  }

  printf("%s\n", cmd);
  ret = EXIT_SUCCESS;

finish:
  fclose(fd);
  return ret;
}

static bool is_empty_file(const char *name) {
  FILE *fd = fopen(name, "rb");
  if (fd == NULL) {
    return true;
  }
  bool is_empty = fread(buf, 1, 1, fd) == 0;
  fclose(fd);
  return is_empty;
}
