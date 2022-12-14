#version 140
    
in vec2 position;

out vec2 uv;

void main() {
    uv = position / 2.0 + vec2(0.5, 0.5);
    gl_Position = vec4(position, 0.0, 1.0);
}