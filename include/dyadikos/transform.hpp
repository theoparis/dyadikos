#pragma once
#include <glm/glm.hpp>
#include <glm/gtx/quaternion.hpp>
#include <glm/gtx/transform.hpp>

namespace dyadikos {
enum class Direction { UP, RIGHT, DOWN, LEFT, FRONT, BACK };

struct Transform {
  glm::vec3 position;
  glm::quat rotation;
  glm::vec3 scale;

  Transform();
  Transform(const glm::vec3& position, const glm::quat& rotation,
	    const glm::vec3& scale);

  [[nodiscard]] auto get_matrix() const -> glm::mat4;
  [[nodiscard]] auto get_direction(Direction direction) const -> glm::vec3;
};
}  // namespace dyadikos
