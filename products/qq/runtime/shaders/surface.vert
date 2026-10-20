// A Shape's pictured surface (Shape.qml, DIRECTION P3.14): where each point is on the shape, in the shape's own units
// and along its own axes, for the fragment shader to lay the pictures by. Qt's material system adds the rest.

VARYING vec3 qqBoxPos;
VARYING vec3 qqBoxNormal;
VARYING vec3 qqAxisX;
VARYING vec3 qqAxisY;
VARYING vec3 qqAxisZ;

void MAIN()
{
    // The mesh's points and normals in the shape's own units: Qt's built-in meshes are a hundred units across, and the
    // shape is scaled. A normal goes by the inverse of a scale.
    qqBoxPos = VERTEX * qqSize;
    qqBoxNormal = NORMAL / qqSize;
    // The shape's axes as the world sees them: only turned, since a normal found in the shape's own units is scaled
    // already. (Qt's normal matrix would undo the scale a second time, and flatten a stretched shape's relief.)
    qqAxisX = normalize(MODEL_MATRIX[0].xyz);
    qqAxisY = normalize(MODEL_MATRIX[1].xyz);
    qqAxisZ = normalize(MODEL_MATRIX[2].xyz);
    POSITION = MODELVIEWPROJECTION_MATRIX * vec4(VERTEX, 1.0);
}
