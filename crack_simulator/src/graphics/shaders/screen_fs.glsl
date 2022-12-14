#version 140
    
out vec4 color;
in vec2 uv;

uniform sampler2D crack_texture;
uniform sampler2D bloom_texture;
uniform vec4 crack_color;
uniform float bloom_mix;
uniform float fade_amt;

void main() {
    color = min((texture(crack_texture, uv) + bloom_mix * texture(bloom_texture, uv)), vec4(1.0)) * crack_color * fade_amt;
}