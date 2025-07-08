// #pragma once
// #include <assimp/postprocess.h>
// #include <assimp/scene.h>
// #include <spdlog/spdlog.h>

// #include <assimp/Importer.hpp>
// #include <string>
// #include <vector>

// namespace dyadikos::model {
// void processNode(aiNode *node, const aiScene *scene,
// 		 std::vector<Vertex> &vertices) {
//   // process all the node's meshes (if any)
//   for (unsigned int i = 0; i < node->mNumMeshes; i++) {
//     aiMesh *mesh = scene->mMeshes[node->mMeshes[i]];

//     for (unsigned int i = 0; i < mesh->mNumFaces; i++) {
//       const aiFace &face = mesh->mFaces[i];

//       for (int j = 0; j < face.mNumIndices; j++) {
// 	const aiVector3D v = mesh->mVertices[face.mIndices[j]];

// 	Vertex vertex{
// 	    glm::vec3(v.x, v.z, v.y),
// 	    glm::vec3(1.0f, 1.0f, 1.0f),
// 	};

// 	vertices.push_back(vertex);
//       }
//     }
//   }
//   // then do the same for each of its children
//   for (unsigned int i = 0; i < node->mNumChildren; i++) {
//     processNode(node->mChildren[i], scene, vertices);
//   }
// }

// auto loadModel(const char *path) -> std::vector<Vertex> {
//   Assimp::Importer importer;
//   auto scene = importer.ReadFile(path, aiProcess_Triangulate);

//   if (!scene || scene->mFlags & AI_SCENE_FLAGS_INCOMPLETE ||
//       !scene->mRootNode) {
//     throw std::runtime_error(importer.GetErrorString());
//   }

//   std::vector<Vertex> vertices;

//   processNode(scene->mRootNode, scene, vertices);

//   return vertices;
// }
// }  // namespace dyadikos::model
