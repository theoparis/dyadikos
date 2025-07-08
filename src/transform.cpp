#include <dyadikos/transform.hpp>

using namespace dyadikos;

Transform::Transform()
    : position(glm::vec3(0.0, 0.0, 0.0)),
      rotation(glm::quat(1.0, 0.0, 0.0, 0.0)),
      scale(glm::vec3(1.0, 1.0, 1.0)) {}

Transform::Transform(const glm::vec3 &position, const glm::quat &rotation,
		     const glm::vec3 &scale)
    : position(position), rotation(rotation), scale(scale) {}

auto Transform::get_matrix() const -> glm::mat4 {
  return glm::translate(position) * glm::toMat4(glm::quat(rotation)) *
	 glm::scale(scale);
}

auto Transform::get_direction(Direction direction) const -> glm::vec3 {
  switch (direction) {
    case Direction::UP:
      return glm::rotate(glm::inverse(rotation), glm::vec3(0.0, 1.0, 0.0));
    case Direction::RIGHT:
      return glm::rotate(glm::inverse(rotation), glm::vec3(1.0, 0.0, 0.0));
    case Direction::DOWN:
      return glm::rotate(glm::inverse(rotation), glm::vec3(0.0, -1.0, 0.0));
    case Direction::LEFT:
      return glm::rotate(glm::inverse(rotation), glm::vec3(-1.0, 0.0, 0.0));
    case Direction::FRONT:
      return glm::rotate(glm::inverse(rotation), glm::vec3(0.0, 0.0, -1.0));
    case Direction::BACK:
      return glm::rotate(glm::inverse(rotation), glm::vec3(0.0, 0.0, 1.0));
    default:
      return glm::vec3(0.0, 0.0, 0.0);
  }
}