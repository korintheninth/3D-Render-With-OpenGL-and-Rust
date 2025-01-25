#version 300 es
precision highp float;

in vec3 FragPos;   
in vec3 Normal;    
in vec2 TexCoords; 

uniform float time;
uniform vec3 cameraPos;
uniform vec3 cameraDir;

uniform sampler2D albedoMap;    
uniform sampler2D aoMap;        
uniform sampler2D metallicSmoothnessMap;  
uniform sampler2D normalMap;    
uniform samplerCube environmentMap;

out vec4 FragColor;

vec3 getNormalFromMap() {
    vec3 tangentNormal = texture(normalMap, TexCoords).xyz * 2.0 - 1.0;
    
    vec3 pos_dx = dFdx(FragPos);
    vec3 pos_dy = dFdy(FragPos);
    vec2 tex_dx = dFdx(TexCoords);
    vec2 tex_dy = dFdy(TexCoords);
    
    vec3 N = normalize(Normal);
    vec3 T = normalize(pos_dx * tex_dy.t - pos_dy * tex_dx.t);
    vec3 B = -normalize(cross(N, T));
    
    mat3 TBN = mat3(T, B, N);
    
    return normalize(TBN * tangentNormal);
}

void main() {
    vec4 albedo = texture(albedoMap, TexCoords);
    float ao = texture(aoMap, TexCoords).r;
    vec4 metallicSmoothness = texture(metallicSmoothnessMap, TexCoords);
    float metallic = metallicSmoothness.r;
    float smoothness = metallicSmoothness.a;
    float roughness = 1.0 - smoothness;
    vec3 normal = getNormalFromMap();

    vec3 viewDir = normalize(cameraPos - FragPos);
    vec3 reflectionDir = reflect(-viewDir, normal);

    vec3 lightColor = vec3(2.0); 

    float innerCutOff = cos(radians(30.0));
    float outerCutOff = cos(radians(35.0));
    
    vec3 fragToLight = normalize(cameraPos - FragPos);
    float theta = dot(fragToLight, normalize(cameraDir));
    float epsilon = innerCutOff - outerCutOff;
    float spotIntensity = clamp((theta - outerCutOff) / epsilon, 0.0, 1.0);

    float diff = max(-dot(normal, fragToLight), 0.0);
    vec3 diffuse = diff * lightColor * albedo.rgb * spotIntensity;

    vec3 halfwayDir = normalize(fragToLight + normalize(cameraDir));
    float spec = pow(max(dot(normal, halfwayDir), 0.0), 128.0 * smoothness);
    vec3 specular = spec * metallic * lightColor * spotIntensity * 2.0;

    vec3 ambient = lightColor * albedo.rgb * ao * 0.05;
    vec3 reflection = texture(environmentMap, reflectionDir).rgb;

    float distance = length(cameraPos - FragPos);
    float attenuation = 1.0 / (1.0 + 0.045 * distance + 0.0075 * distance * distance);

    vec3 finalColor = ambient + (diffuse + specular) * attenuation;
    finalColor = mix(finalColor, reflection, metallic * smoothness);

    FragColor = vec4(finalColor, albedo.a);
}