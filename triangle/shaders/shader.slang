// Hardcoded vertex positions and colors for a triangle. This means we don't need to pass in any vertex buffers or index buffers, YET!
static float2 positions[3] = float2[](
    float2(0.0, -0.5),
    float2(0.5, 0.5),
    float2(-0.5, 0.5)
);

// Hardcoded colors for each vertex of the triangle. These colors are used in the vertex shader to color the triangle.
static float3 colors[3] = float3[](
    float3(1.0, 0.0, 0.0),
    float3(0.0, 1.0, 0.0),
    float3(0.0, 0.0, 1.0)
);
 
// Custom vertex output structure to pass data from the vertex shader to the fragment shader.
struct VertexOutput {
    float3 color;
    float4 sv_position : SV_Position;
};

[shader("vertex")]
VertexOutput vertMain(uint vid : SV_VertexID) {
    VertexOutput output;

    output.sv_position = float4(positions[vid], 0.0, 1.0);
    output.color = colors[vid];

    return output;
}

[shader("fragment")]
float4 fragMain(VertexOutput inVert) : SV_Target
{
    float3 color = inVert.color;
    return float4(color, 1.0);
}