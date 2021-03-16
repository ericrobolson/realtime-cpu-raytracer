out vec4 FragColor;
  
in vec4 v_color;
in vec2 v_tex_coord;

uniform sampler2D s_tex0;

void main()
{
    FragColor = texture(s_tex0, v_tex_coord) * v_color;
}
