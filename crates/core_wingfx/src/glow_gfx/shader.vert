layout (location = 0) in vec3 aPos;
layout (location = 1) in vec4 aColor;
layout (location = 2) in vec2 aTexCoord;

out vec4 v_color;
out vec2 v_tex_coord;

void main()
{
    v_color = aColor;
    v_tex_coord = aTexCoord;
    gl_Position = vec4(aPos, 1.0);
}
