#version 460 core
uniform sampler2D diffuseTexture;
out vec4 FragColor;
in vec2 texCoord;
void main() {
    FragColor = vec4(1, 0, 0, 1); //texture(diffuseTexture, texCoord);
}