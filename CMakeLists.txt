cmake_minimum_required(VERSION 3.1...3.21)

file(STRINGS "VERSION" ver)

project(
  shell-snoop
  VERSION ${ver}
  LANGUAGES C)

add_executable(shell-snoop src/shell-snoop.c)

target_link_libraries(shell-snoop cap-ng)

install(TARGETS shell-snoop DESTINATION bin)
