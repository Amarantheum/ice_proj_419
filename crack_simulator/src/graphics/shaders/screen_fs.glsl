#version 140
    
out vec4 color;
in vec2 uv;

uniform sampler2D crack_texture;
uniform vec4 crack_color;

void main() {
    color = texture(crack_texture, uv) * crack_color;
}