#version 450

// Texture Coordinates received from the Vertex Shader as input
layout(location = 0) in vec2 TexCoords;
layout(location = 1) in vec4 Color;

// Color value returned as an output. This is usually the case for a fragment shader
layout(location = 0) out vec4 color;

layout(set = 1, binding = 0) uniform sampler2D image;

void main()
{    
    color = Color * texture(image, TexCoords);
    // color = Color;
}  