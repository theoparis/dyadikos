#include "window.h"
#include <stdio.h>
#include <stdlib.h>

int main() {
  struct dyadikos_window window = {};
  if (!dyadikos_window_new(&window, 1280, 720)) {
    fprintf(stderr, "Can't connect to display\n");

    return EXIT_FAILURE;
  }

  while (true) {
    dyadikos_window_update(&window);
  }

  return 0;
}
