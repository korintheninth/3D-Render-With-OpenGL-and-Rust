#version 300 es
precision highp float;

in vec3 FragPos;   // Fragment position in world space
in vec3 Normal;    // Fragment normal
in vec2 TexCoords; // Fragment texture coordinates
uniform vec3 lightPos;  // Camera position as light source

out vec4 FragColor;

// Light properties
vec3 lightColor = vec3(1.0, 1.0, 1.0);
vec3 objectColor = vec3(100.0/255.0, 149.0/255.0, 237.0/255.0); 

// Material properties
float ambientStrength = 0.2;
float specularStrength = 1.0;
float shininess = 32.0;

void main() {
    // Ambient
    vec3 ambient = ambientStrength * lightColor;

    // Diffuse
    vec3 norm = normalize(Normal);
    vec3 lightDir = normalize(lightPos - FragPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;

    // Specular
    vec3 viewDir = normalize(lightPos - FragPos); // View direction is same as light direction
    vec3 reflectDir = reflect(-lightDir, norm);
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);
    vec3 specular = specularStrength * spec * lightColor;

    // Final color
    vec3 result = (ambient + diffuse + specular) * objectColor;
    FragColor = vec4(result, 1.0);
}