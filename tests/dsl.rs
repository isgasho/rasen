#[allow(clippy::needless_pass_by_value)]
pub fn basic_vert(
    a_pos: Value<Vec3>,
    a_normal: Value<Vec3>,
    a_uv: Value<Vec2>,
    projection: Value<Mat4>,
    view: Value<Mat4>,
    model: Value<Mat4>,
) -> (Value<Vec4>, Value<Vec4>, Value<Vec2>) {
    let mvp = projection * view * model.clone();

    let v_pos = mvp * vec4(index(&a_pos, 0), index(&a_pos, 1), index(&a_pos, 2), 1.0f32);

    let v_norm = model
        * vec4(
            index(&a_normal, 0),
            index(&a_normal, 1),
            index(&a_normal, 2),
            1.0f32,
        );

    (v_pos, v_norm, a_uv)
}

pub fn basic_frag(
    a_normal: Value<Vec3>,
    a_uv: Value<Vec2>,
    material: Value<Sampler>,
) -> Value<Vec4> {
    let normal = normalize(a_normal);
    let light = vec3(0.3f32, -0.5f32, 0.2f32);
    let color = sample(material, a_uv);

    clamp(dot(normal, light), 0.1f32, 1.0f32) * color
}

fn func(input: Value<Float>) -> Value<Float> {
    input
}

pub fn functions(a_input: Value<Float>) -> Value<Float> {
    func(a_input)
}
