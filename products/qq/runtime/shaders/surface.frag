// A Shape's pictured surface (Shape.qml, DIRECTION P3.14). Its picture and its relief are laid along the shape's own
// three axes, one copy every qqImageSize units, and blended where a surface faces between them: a box gets each face
// square-on, and a long wall as many copies as it is long. Lighting, shadows and the sky's light are Qt's own (a Shaded
// custom material), exactly as on a plain shape.

VARYING vec3 qqBoxPos;
VARYING vec3 qqBoxNormal;
VARYING vec3 qqAxisX;
VARYING vec3 qqAxisY;
VARYING vec3 qqAxisZ;

// A picture's colours are sRGB, and lighting works in linear light: the conversion Qt's own materials use.
vec3 qqLinear(vec3 rgb)
{
    return rgb * (rgb * (rgb * 0.305306011 + vec3(0.682171111)) + vec3(0.012522878));
}

void MAIN()
{
    vec3 n = normalize(qqBoxNormal);
    // How much each axis's projection counts: the one the surface faces, and a narrow blend between them.
    vec3 w = pow(abs(n), vec3(4.0));
    w /= max(w.x + w.y + w.z, 0.0001);
    // Each face seen from outside, upright: x faces run along -z or +z, z faces along +x or -x, and y faces along x.
    float sx = n.x < 0.0 ? -1.0 : 1.0;
    float sy = n.y < 0.0 ? -1.0 : 1.0;
    float sz = n.z < 0.0 ? -1.0 : 1.0;
    vec3 p = qqBoxPos / qqImageSize;
    vec2 uvX = vec2(-p.z * sx, p.y);
    vec2 uvY = vec2(p.x, -p.z * sy);
    vec2 uvZ = vec2(p.x * sz, p.y);

    vec3 colour = qqTint.rgb;
    if (qqHasPicture) {
        vec3 painted = texture(qqPicture, uvX).rgb * w.x + texture(qqPicture, uvY).rgb * w.y
                     + texture(qqPicture, uvZ).rgb * w.z;
        colour *= qqLinear(painted);
    }
    BASE_COLOR = vec4(colour, qqTint.a);
    METALNESS = qqMetal;
    ROUGHNESS = qqRough;
    EMISSIVE_COLOR = qqGlow;

    if (qqHasRelief) {
        // Each projection's relief, tilted along that projection's own across and up, added to the surface's normal.
        vec2 rX = texture(qqRelief, uvX).xy * 2.0 - 1.0;
        vec2 rY = texture(qqRelief, uvY).xy * 2.0 - 1.0;
        vec2 rZ = texture(qqRelief, uvZ).xy * 2.0 - 1.0;
        vec3 tilt = w.x * (vec3(0.0, 0.0, -sx) * rX.x + vec3(0.0, 1.0, 0.0) * rX.y)
                  + w.y * (vec3(1.0, 0.0, 0.0) * rY.x + vec3(0.0, 0.0, -sy) * rY.y)
                  + w.z * (vec3(sz, 0.0, 0.0) * rZ.x + vec3(0.0, 1.0, 0.0) * rZ.y);
        vec3 objectNormal = normalize(n + tilt);
        NORMAL = normalize(mat3(qqAxisX, qqAxisY, qqAxisZ) * objectNormal);
    }
}
