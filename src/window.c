#include "window.h"

bool dyadikos_window_new(struct dyadikos_window *window, int width,
                         int height) {
#if defined(_WIN32)
  window->window =
      CreateWindowEx(0, "WindowClass", "Dyadikos", WS_OVERLAPPEDWINDOW,
                     CW_USEDEFAULT, CW_USEDEFAULT, width, height, nullptr,
                     nullptr, GetModuleHandle(nullptr), nullptr);
  window->display = GetDC(hwnd);
  ShowWindow(hwnd, SW_SHOW);
  UpdateWindow(hwnd);
#elif defined(__linux)
  window->connection = xcb_connect(nullptr, nullptr);

  if (xcb_connection_has_error(window->connection)) {
    return false;
  }

  window->screen =
      xcb_setup_roots_iterator(xcb_get_setup(window->connection)).data;

  uint32_t mask;
  uint32_t values[2];

  mask = XCB_CW_BACK_PIXEL | XCB_CW_EVENT_MASK;
  values[0] = window->screen->white_pixel;
  values[1] = XCB_EVENT_MASK_EXPOSURE | XCB_EVENT_MASK_KEY_PRESS;

  window->window = xcb_generate_id(window->connection);
  xcb_create_window(window->connection, XCB_COPY_FROM_PARENT, window->window,
                    window->screen->root, 0, 0, 640, 480, 0,
                    XCB_WINDOW_CLASS_INPUT_OUTPUT, window->screen->root_visual,
                    mask, values);
  xcb_flush(window->connection);
#endif

  return true;
}

void dyadikos_window_update(struct dyadikos_window *window) {}
