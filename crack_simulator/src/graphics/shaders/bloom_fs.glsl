#version 140
    
out vec4 color;
in vec2 uv;

uniform sampler2D crack_texture;
uniform vec4 crack_color;
uniform float bloom_size;
uniform uint width;
uniform uint height;

void main() {
    vec4 sum = vec4(0);
    for (int i = -bloom_size / 2; i < bloom_size / 2; i++) {
        for (int j = -bloom_size / 2; i < bloom_size / 2; j++) {

            sum += texture(crack_texture, vec2(uv.x + i /))
        }
    }
    color = texture(crack_texture, uv) * crack_color;
}