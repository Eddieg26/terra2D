#version 450

// Position and Texture Cooridnates stored in a single vec4 for convenience
layout (location = 0) in vec4 vertex; // <vec2 position, vec2 texCoords>

// Output the UV Texture Coordinates
layout(location = 0) out vec2 TexCoords;
layout(location = 1) out vec4 Color;

// Uniforms are so named because they do not change from one shader invocation to the next within a particular rendering call.
layout(set = 0, binding = 0) uniform PerCamera {
    mat4 projection;
    mat4 view;
}camera;

// Sprite transform
layout(push_constant) uniform PerObject {
    mat4 model;
    vec4 color;
}object;


void main()
{
    TexCoords = vertex.zw;
    Color = object.color;
    gl_Position = camera.projection * camera.view * object.model * vec4(vertex.xy, 0.0, 1.0);
    gl_Position = vec4(vertex.xy, 0.0, 1.0);
    // gl_position is used to store the position of the current vertex
    // the value of this variable is used in proceeding pipeline stages
}
