#include <cmath>
#include <dyadikos/app.hpp>
#include <iostream>
#include <print>

int main() {
  try {
    dyadikos::App app("Rainbow Window");

    app.Run([&app](double time) {
      auto const &frame = app.BeginRender(
	  glm::vec4(std::sin(time * 0.001) * 0.5f + 0.5f, 0.0f,
		    std::cos(time * 0.001) * 0.5f + 0.5f, 1.0f));

      app.EndRender(frame);
    });
  } catch (const dyadikos::SDLException &e) {
    std::print(std::cerr, "SDL Error: {}\n", e.what());
    return EXIT_FAILURE;
  }

  return EXIT_SUCCESS;
}