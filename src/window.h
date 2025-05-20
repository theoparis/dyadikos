#if defined(_WIN32)
#include <Windows.h>
#elif defined(__linux__)
#include <xcb/xcb.h>
#endif

struct dyadikos_window {
#ifdef _WIN32
  HWND window;
  HDC display;
#elif defined(__linux__)
  xcb_connection_t *connection;
  xcb_screen_t *screen;
  xcb_window_t window;
#endif
};

bool dyadikos_window_new(struct dyadikos_window *window, int width, int height);
void dyadikos_window_update(struct dyadikos_window *window);
