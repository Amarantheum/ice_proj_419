#version 140
    
out vec4 color;
in vec2 uv;

uniform sampler2D crack_texture;
uniform vec4 crack_color;
uniform int bloom_size;
uniform float scale;

void main() {
    vec4 sum = vec4(0);
    for (int i = -bloom_size / 2; i < bloom_size / 2; i++) {
        for (int j = -bloom_size / 2; j < bloom_size / 2; j++) {
            float dist = sqrt(pow(float(i) / float(bloom_size), 2) + pow(float(j) / float(bloom_size), 2));
            float n = max(1 - dist, 0.0);
            sum += n * texture(crack_texture, vec2(uv.x + (float(i) / scale), uv.y + (float(j) / scale)));
        }
    }
    color = sum / (bloom_size * bloom_size) * crack_color;
}