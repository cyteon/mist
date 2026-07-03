// Generated from vanilla 1.21 worldgen details zdump

pub struct Spline {
    pub coord: usize,
    pub loc: &'static [f32],
    pub der: &'static [f32],
    pub val: &'static [Node],
}

pub enum Node {
    C(f32),
    S(&'static Spline),
}

static S0: Spline = Spline {
    coord: 0,
    loc: &[
        -1.1f32, -1.02f32, -0.51f32, -0.44f32, -0.18f32, -0.16f32, -0.15f32, -0.1f32, 0.25f32,
        1.0f32,
    ],
    der: &[
        0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32,
    ],
    val: &[
        Node::C(0.044f32),
        Node::C(-0.2222f32),
        Node::C(-0.2222f32),
        Node::C(-0.12f32),
        Node::C(-0.12f32),
        Node::S(&S1),
        Node::S(&S9),
        Node::S(&S17),
        Node::S(&S25),
        Node::S(&S39),
    ],
};
static S1: Spline = Spline {
    coord: 1,
    loc: &[
        -0.85f32, -0.7f32, -0.4f32, -0.35f32, -0.1f32, 0.2f32, 0.7f32,
    ],
    der: &[0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32],
    val: &[
        Node::S(&S2),
        Node::S(&S3),
        Node::S(&S4),
        Node::S(&S5),
        Node::S(&S6),
        Node::S(&S7),
        Node::S(&S8),
    ],
};
static S2: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, 1.0f32],
    der: &[0.38940096f32, 0.38940096f32],
    val: &[Node::C(-0.08880186f32), Node::C(0.69000006f32)],
};
static S3: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, 1.0f32],
    der: &[0.37788022f32, 0.37788022f32],
    val: &[Node::C(-0.115760356f32), Node::C(0.6400001f32)],
};
static S4: Spline = Spline {
    coord: 2,
    loc: &[
        -1.0f32,
        -0.75f32,
        -0.65f32,
        0.5954547f32,
        0.6054547f32,
        1.0f32,
    ],
    der: &[0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.2534563f32, 0.2534563f32],
    val: &[
        Node::C(-0.2222f32),
        Node::C(-0.2222f32),
        Node::C(0.0f32),
        Node::C(2.9802322e-08f32),
        Node::C(2.9802322e-08f32),
        Node::C(0.100000024f32),
    ],
};
static S5: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.0f32, 0.007000001f32],
    val: &[
        Node::C(-0.3f32),
        Node::C(0.05f32),
        Node::C(0.05f32),
        Node::C(0.05f32),
        Node::C(0.060000002f32),
    ],
};
static S6: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.1f32, 0.007000001f32],
    val: &[
        Node::C(-0.15f32),
        Node::C(0.0f32),
        Node::C(0.0f32),
        Node::C(0.05f32),
        Node::C(0.060000002f32),
    ],
};
static S7: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32],
    val: &[
        Node::C(-0.15f32),
        Node::C(0.0f32),
        Node::C(0.0f32),
        Node::C(0.0f32),
        Node::C(0.0f32),
    ],
};
static S8: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.0f32, 0.0f32, 0.0f32, 0.06f32, 0.0f32],
    val: &[
        Node::C(-0.02f32),
        Node::C(-0.03f32),
        Node::C(-0.03f32),
        Node::C(0.0f32),
        Node::C(0.0f32),
    ],
};
static S9: Spline = Spline {
    coord: 1,
    loc: &[
        -0.85f32, -0.7f32, -0.4f32, -0.35f32, -0.1f32, 0.2f32, 0.7f32,
    ],
    der: &[0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32],
    val: &[
        Node::S(&S10),
        Node::S(&S11),
        Node::S(&S12),
        Node::S(&S13),
        Node::S(&S14),
        Node::S(&S15),
        Node::S(&S16),
    ],
};
static S10: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, 1.0f32],
    der: &[0.38940096f32, 0.38940096f32],
    val: &[Node::C(-0.08880186f32), Node::C(0.69000006f32)],
};
static S11: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, 1.0f32],
    der: &[0.37788022f32, 0.37788022f32],
    val: &[Node::C(-0.115760356f32), Node::C(0.6400001f32)],
};
static S12: Spline = Spline {
    coord: 2,
    loc: &[
        -1.0f32,
        -0.75f32,
        -0.65f32,
        0.5954547f32,
        0.6054547f32,
        1.0f32,
    ],
    der: &[0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.2534563f32, 0.2534563f32],
    val: &[
        Node::C(-0.2222f32),
        Node::C(-0.2222f32),
        Node::C(0.0f32),
        Node::C(2.9802322e-08f32),
        Node::C(2.9802322e-08f32),
        Node::C(0.100000024f32),
    ],
};
static S13: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.0f32, 0.007000001f32],
    val: &[
        Node::C(-0.3f32),
        Node::C(0.05f32),
        Node::C(0.05f32),
        Node::C(0.05f32),
        Node::C(0.060000002f32),
    ],
};
static S14: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.1f32, 0.007000001f32],
    val: &[
        Node::C(-0.15f32),
        Node::C(0.0f32),
        Node::C(0.0f32),
        Node::C(0.05f32),
        Node::C(0.060000002f32),
    ],
};
static S15: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32],
    val: &[
        Node::C(-0.15f32),
        Node::C(0.0f32),
        Node::C(0.0f32),
        Node::C(0.0f32),
        Node::C(0.0f32),
    ],
};
static S16: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.0f32, 0.0f32, 0.0f32, 0.06f32, 0.0f32],
    val: &[
        Node::C(-0.02f32),
        Node::C(-0.03f32),
        Node::C(-0.03f32),
        Node::C(0.0f32),
        Node::C(0.0f32),
    ],
};
static S17: Spline = Spline {
    coord: 1,
    loc: &[
        -0.85f32, -0.7f32, -0.4f32, -0.35f32, -0.1f32, 0.2f32, 0.7f32,
    ],
    der: &[0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32],
    val: &[
        Node::S(&S18),
        Node::S(&S19),
        Node::S(&S20),
        Node::S(&S21),
        Node::S(&S22),
        Node::S(&S23),
        Node::S(&S24),
    ],
};
static S18: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, 1.0f32],
    der: &[0.38940096f32, 0.38940096f32],
    val: &[Node::C(-0.08880186f32), Node::C(0.69000006f32)],
};
static S19: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, 1.0f32],
    der: &[0.37788022f32, 0.37788022f32],
    val: &[Node::C(-0.115760356f32), Node::C(0.6400001f32)],
};
static S20: Spline = Spline {
    coord: 2,
    loc: &[
        -1.0f32,
        -0.75f32,
        -0.65f32,
        0.5954547f32,
        0.6054547f32,
        1.0f32,
    ],
    der: &[0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.2534563f32, 0.2534563f32],
    val: &[
        Node::C(-0.2222f32),
        Node::C(-0.2222f32),
        Node::C(0.0f32),
        Node::C(2.9802322e-08f32),
        Node::C(2.9802322e-08f32),
        Node::C(0.100000024f32),
    ],
};
static S21: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.0f32, 0.007000001f32],
    val: &[
        Node::C(-0.25f32),
        Node::C(0.05f32),
        Node::C(0.05f32),
        Node::C(0.05f32),
        Node::C(0.060000002f32),
    ],
};
static S22: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.01f32, 0.01f32, 0.094000004f32, 0.007000001f32],
    val: &[
        Node::C(-0.1f32),
        Node::C(0.001f32),
        Node::C(0.003f32),
        Node::C(0.05f32),
        Node::C(0.060000002f32),
    ],
};
static S23: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.04f32, 0.049f32],
    val: &[
        Node::C(-0.1f32),
        Node::C(0.01f32),
        Node::C(0.01f32),
        Node::C(0.03f32),
        Node::C(0.1f32),
    ],
};
static S24: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.0f32, 0.0f32, 0.0f32, 0.12f32, 0.049f32],
    val: &[
        Node::C(-0.02f32),
        Node::C(-0.03f32),
        Node::C(-0.03f32),
        Node::C(0.03f32),
        Node::C(0.1f32),
    ],
};
static S25: Spline = Spline {
    coord: 1,
    loc: &[
        -0.85f32, -0.7f32, -0.4f32, -0.35f32, -0.1f32, 0.2f32, 0.4f32, 0.45f32, 0.55f32, 0.58f32,
        0.7f32,
    ],
    der: &[
        0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32,
    ],
    val: &[
        Node::S(&S26),
        Node::S(&S27),
        Node::S(&S28),
        Node::S(&S29),
        Node::S(&S30),
        Node::S(&S31),
        Node::S(&S32),
        Node::S(&S33),
        Node::S(&S35),
        Node::S(&S37),
        Node::S(&S38),
    ],
};
static S26: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, 0.0f32, 1.0f32],
    der: &[0.0f32, 0.5138249f32, 0.5138249f32],
    val: &[
        Node::C(0.20235021f32),
        Node::C(0.7161751f32),
        Node::C(1.23f32),
    ],
};
static S27: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, 0.0f32, 1.0f32],
    der: &[0.0f32, 0.43317974f32, 0.43317974f32],
    val: &[Node::C(0.2f32), Node::C(0.44682026f32), Node::C(0.88f32)],
};
static S28: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, 0.0f32, 1.0f32],
    der: &[0.0f32, 0.3917051f32, 0.3917051f32],
    val: &[
        Node::C(0.2f32),
        Node::C(0.30829495f32),
        Node::C(0.70000005f32),
    ],
};
static S29: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.0f32, 0.049000014f32],
    val: &[
        Node::C(-0.25f32),
        Node::C(0.35f32),
        Node::C(0.35f32),
        Node::C(0.35f32),
        Node::C(0.42000002f32),
    ],
};
static S30: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.07f32, 0.07f32, 0.658f32, 0.049000014f32],
    val: &[
        Node::C(-0.1f32),
        Node::C(0.0069999998f32),
        Node::C(0.021f32),
        Node::C(0.35f32),
        Node::C(0.42000002f32),
    ],
};
static S31: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.04f32, 0.049f32],
    val: &[
        Node::C(-0.1f32),
        Node::C(0.01f32),
        Node::C(0.01f32),
        Node::C(0.03f32),
        Node::C(0.1f32),
    ],
};
static S32: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.04f32, 0.049f32],
    val: &[
        Node::C(-0.1f32),
        Node::C(0.01f32),
        Node::C(0.01f32),
        Node::C(0.03f32),
        Node::C(0.1f32),
    ],
};
static S33: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32],
    der: &[0.0f32, 0.0f32, 0.0f32],
    val: &[Node::C(-0.1f32), Node::S(&S34), Node::C(0.17f32)],
};
static S34: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.04f32, 0.049f32],
    val: &[
        Node::C(-0.1f32),
        Node::C(0.01f32),
        Node::C(0.01f32),
        Node::C(0.03f32),
        Node::C(0.1f32),
    ],
};
static S35: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32],
    der: &[0.0f32, 0.0f32, 0.0f32],
    val: &[Node::C(-0.1f32), Node::S(&S36), Node::C(0.17f32)],
};
static S36: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.04f32, 0.049f32],
    val: &[
        Node::C(-0.1f32),
        Node::C(0.01f32),
        Node::C(0.01f32),
        Node::C(0.03f32),
        Node::C(0.1f32),
    ],
};
static S37: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.04f32, 0.049f32],
    val: &[
        Node::C(-0.1f32),
        Node::C(0.01f32),
        Node::C(0.01f32),
        Node::C(0.03f32),
        Node::C(0.1f32),
    ],
};
static S38: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.0f32, 0.0f32, 0.0f32, 0.12f32, 0.049f32],
    val: &[
        Node::C(-0.02f32),
        Node::C(-0.03f32),
        Node::C(-0.03f32),
        Node::C(0.03f32),
        Node::C(0.1f32),
    ],
};
static S39: Spline = Spline {
    coord: 1,
    loc: &[
        -0.85f32, -0.7f32, -0.4f32, -0.35f32, -0.1f32, 0.2f32, 0.4f32, 0.45f32, 0.55f32, 0.58f32,
        0.7f32,
    ],
    der: &[
        0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32,
    ],
    val: &[
        Node::S(&S40),
        Node::S(&S41),
        Node::S(&S42),
        Node::S(&S43),
        Node::S(&S44),
        Node::S(&S45),
        Node::S(&S46),
        Node::S(&S47),
        Node::S(&S49),
        Node::S(&S51),
        Node::S(&S52),
    ],
};
static S40: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, 0.0f32, 1.0f32],
    der: &[0.0f32, 0.5760369f32, 0.5760369f32],
    val: &[
        Node::C(0.34792626f32),
        Node::C(0.9239631f32),
        Node::C(1.5f32),
    ],
};
static S41: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, 0.0f32, 1.0f32],
    der: &[0.0f32, 0.4608295f32, 0.4608295f32],
    val: &[Node::C(0.2f32), Node::C(0.5391705f32), Node::C(1.0f32)],
};
static S42: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, 0.0f32, 1.0f32],
    der: &[0.0f32, 0.4608295f32, 0.4608295f32],
    val: &[Node::C(0.2f32), Node::C(0.5391705f32), Node::C(1.0f32)],
};
static S43: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.0f32, 0.070000015f32],
    val: &[
        Node::C(-0.2f32),
        Node::C(0.5f32),
        Node::C(0.5f32),
        Node::C(0.5f32),
        Node::C(0.6f32),
    ],
};
static S44: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[
        0.5f32,
        0.099999994f32,
        0.099999994f32,
        0.94f32,
        0.070000015f32,
    ],
    val: &[
        Node::C(-0.05f32),
        Node::C(0.01f32),
        Node::C(0.03f32),
        Node::C(0.5f32),
        Node::C(0.6f32),
    ],
};
static S45: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.04f32, 0.049f32],
    val: &[
        Node::C(-0.05f32),
        Node::C(0.01f32),
        Node::C(0.01f32),
        Node::C(0.03f32),
        Node::C(0.1f32),
    ],
};
static S46: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.04f32, 0.049f32],
    val: &[
        Node::C(-0.05f32),
        Node::C(0.01f32),
        Node::C(0.01f32),
        Node::C(0.03f32),
        Node::C(0.1f32),
    ],
};
static S47: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32],
    der: &[0.0f32, 0.0f32, 0.0f32],
    val: &[Node::C(-0.05f32), Node::S(&S48), Node::C(0.17f32)],
};
static S48: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.04f32, 0.049f32],
    val: &[
        Node::C(-0.05f32),
        Node::C(0.01f32),
        Node::C(0.01f32),
        Node::C(0.03f32),
        Node::C(0.1f32),
    ],
};
static S49: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32],
    der: &[0.0f32, 0.0f32, 0.0f32],
    val: &[Node::C(-0.05f32), Node::S(&S50), Node::C(0.17f32)],
};
static S50: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.04f32, 0.049f32],
    val: &[
        Node::C(-0.05f32),
        Node::C(0.01f32),
        Node::C(0.01f32),
        Node::C(0.03f32),
        Node::C(0.1f32),
    ],
};
static S51: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.5f32, 0.0f32, 0.0f32, 0.04f32, 0.049f32],
    val: &[
        Node::C(-0.05f32),
        Node::C(0.01f32),
        Node::C(0.01f32),
        Node::C(0.03f32),
        Node::C(0.1f32),
    ],
};
static S52: Spline = Spline {
    coord: 2,
    loc: &[-1.0f32, -0.4f32, 0.0f32, 0.4f32, 1.0f32],
    der: &[0.015f32, 0.0f32, 0.0f32, 0.04f32, 0.049f32],
    val: &[
        Node::C(-0.02f32),
        Node::C(0.01f32),
        Node::C(0.01f32),
        Node::C(0.03f32),
        Node::C(0.1f32),
    ],
};
static S53: Spline = Spline {
    coord: 0,
    loc: &[-0.19f32, -0.15f32, -0.1f32, 0.03f32, 0.06f32],
    der: &[0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32],
    val: &[
        Node::C(3.95f32),
        Node::S(&S54),
        Node::S(&S65),
        Node::S(&S76),
        Node::S(&S87),
    ],
};
static S54: Spline = Spline {
    coord: 1,
    loc: &[
        -0.6f32, -0.5f32, -0.35f32, -0.25f32, -0.1f32, 0.03f32, 0.35f32, 0.45f32, 0.55f32, 0.62f32,
    ],
    der: &[
        0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32,
    ],
    val: &[
        Node::S(&S55),
        Node::S(&S56),
        Node::S(&S57),
        Node::S(&S58),
        Node::S(&S59),
        Node::S(&S60),
        Node::C(6.25f32),
        Node::S(&S61),
        Node::S(&S63),
        Node::C(6.25f32),
    ],
};
static S55: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(6.25f32)],
};
static S56: Spline = Spline {
    coord: 3,
    loc: &[-0.05f32, 0.05f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(2.67f32)],
};
static S57: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(6.25f32)],
};
static S58: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(6.25f32)],
};
static S59: Spline = Spline {
    coord: 3,
    loc: &[-0.05f32, 0.05f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(2.67f32), Node::C(6.3f32)],
};
static S60: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(6.25f32)],
};
static S61: Spline = Spline {
    coord: 2,
    loc: &[-0.9f32, -0.69f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.25f32), Node::S(&S62)],
};
static S62: Spline = Spline {
    coord: 3,
    loc: &[0.0f32, 0.1f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.25f32), Node::C(0.625f32)],
};
static S63: Spline = Spline {
    coord: 2,
    loc: &[-0.9f32, -0.69f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.25f32), Node::S(&S64)],
};
static S64: Spline = Spline {
    coord: 3,
    loc: &[0.0f32, 0.1f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.25f32), Node::C(0.625f32)],
};
static S65: Spline = Spline {
    coord: 1,
    loc: &[
        -0.6f32, -0.5f32, -0.35f32, -0.25f32, -0.1f32, 0.03f32, 0.35f32, 0.45f32, 0.55f32, 0.62f32,
    ],
    der: &[
        0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32,
    ],
    val: &[
        Node::S(&S66),
        Node::S(&S67),
        Node::S(&S68),
        Node::S(&S69),
        Node::S(&S70),
        Node::S(&S71),
        Node::C(5.47f32),
        Node::S(&S72),
        Node::S(&S74),
        Node::C(5.47f32),
    ],
};
static S66: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(5.47f32)],
};
static S67: Spline = Spline {
    coord: 3,
    loc: &[-0.05f32, 0.05f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(2.67f32)],
};
static S68: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(5.47f32)],
};
static S69: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(5.47f32)],
};
static S70: Spline = Spline {
    coord: 3,
    loc: &[-0.05f32, 0.05f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(2.67f32), Node::C(6.3f32)],
};
static S71: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(5.47f32)],
};
static S72: Spline = Spline {
    coord: 2,
    loc: &[-0.9f32, -0.69f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(5.47f32), Node::S(&S73)],
};
static S73: Spline = Spline {
    coord: 3,
    loc: &[0.0f32, 0.1f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(5.47f32), Node::C(0.625f32)],
};
static S74: Spline = Spline {
    coord: 2,
    loc: &[-0.9f32, -0.69f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(5.47f32), Node::S(&S75)],
};
static S75: Spline = Spline {
    coord: 3,
    loc: &[0.0f32, 0.1f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(5.47f32), Node::C(0.625f32)],
};
static S76: Spline = Spline {
    coord: 1,
    loc: &[
        -0.6f32, -0.5f32, -0.35f32, -0.25f32, -0.1f32, 0.03f32, 0.35f32, 0.45f32, 0.55f32, 0.62f32,
    ],
    der: &[
        0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32,
    ],
    val: &[
        Node::S(&S77),
        Node::S(&S78),
        Node::S(&S79),
        Node::S(&S80),
        Node::S(&S81),
        Node::S(&S82),
        Node::C(5.08f32),
        Node::S(&S83),
        Node::S(&S85),
        Node::C(5.08f32),
    ],
};
static S77: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(5.08f32)],
};
static S78: Spline = Spline {
    coord: 3,
    loc: &[-0.05f32, 0.05f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(2.67f32)],
};
static S79: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(5.08f32)],
};
static S80: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(5.08f32)],
};
static S81: Spline = Spline {
    coord: 3,
    loc: &[-0.05f32, 0.05f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(2.67f32), Node::C(6.3f32)],
};
static S82: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(5.08f32)],
};
static S83: Spline = Spline {
    coord: 2,
    loc: &[-0.9f32, -0.69f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(5.08f32), Node::S(&S84)],
};
static S84: Spline = Spline {
    coord: 3,
    loc: &[0.0f32, 0.1f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(5.08f32), Node::C(0.625f32)],
};
static S85: Spline = Spline {
    coord: 2,
    loc: &[-0.9f32, -0.69f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(5.08f32), Node::S(&S86)],
};
static S86: Spline = Spline {
    coord: 3,
    loc: &[0.0f32, 0.1f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(5.08f32), Node::C(0.625f32)],
};
static S87: Spline = Spline {
    coord: 1,
    loc: &[
        -0.6f32, -0.5f32, -0.35f32, -0.25f32, -0.1f32, 0.03f32, 0.05f32, 0.4f32, 0.45f32, 0.55f32,
        0.58f32,
    ],
    der: &[
        0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32,
    ],
    val: &[
        Node::S(&S88),
        Node::S(&S89),
        Node::S(&S90),
        Node::S(&S91),
        Node::S(&S92),
        Node::S(&S93),
        Node::S(&S94),
        Node::S(&S96),
        Node::S(&S98),
        Node::S(&S100),
        Node::C(4.69f32),
    ],
};
static S88: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(4.69f32)],
};
static S89: Spline = Spline {
    coord: 3,
    loc: &[-0.05f32, 0.05f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(2.67f32)],
};
static S90: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(4.69f32)],
};
static S91: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(4.69f32)],
};
static S92: Spline = Spline {
    coord: 3,
    loc: &[-0.05f32, 0.05f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(2.67f32), Node::C(6.3f32)],
};
static S93: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(4.69f32)],
};
static S94: Spline = Spline {
    coord: 2,
    loc: &[0.45f32, 0.7f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::S(&S95), Node::C(1.56f32)],
};
static S95: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(4.69f32)],
};
static S96: Spline = Spline {
    coord: 2,
    loc: &[0.45f32, 0.7f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::S(&S97), Node::C(1.56f32)],
};
static S97: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(4.69f32)],
};
static S98: Spline = Spline {
    coord: 2,
    loc: &[-0.7f32, -0.15f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::S(&S99), Node::C(1.37f32)],
};
static S99: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(4.69f32)],
};
static S100: Spline = Spline {
    coord: 2,
    loc: &[-0.7f32, -0.15f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::S(&S101), Node::C(1.37f32)],
};
static S101: Spline = Spline {
    coord: 3,
    loc: &[-0.2f32, 0.2f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(6.3f32), Node::C(4.69f32)],
};
static S102: Spline = Spline {
    coord: 0,
    loc: &[-0.11f32, 0.03f32, 0.65f32],
    der: &[0.0f32, 0.0f32, 0.0f32],
    val: &[Node::C(0.0f32), Node::S(&S103), Node::S(&S110)],
};
static S103: Spline = Spline {
    coord: 1,
    loc: &[-1.0f32, -0.78f32, -0.5775f32, -0.375f32],
    der: &[0.0f32, 0.0f32, 0.0f32, 0.0f32],
    val: &[
        Node::S(&S104),
        Node::S(&S106),
        Node::S(&S108),
        Node::C(0.0f32),
    ],
};
static S104: Spline = Spline {
    coord: 2,
    loc: &[0.19999999f32, 0.44999996f32, 1.0f32],
    der: &[0.0f32, 0.0f32, 0.0f32],
    val: &[Node::C(0.0f32), Node::C(0.0f32), Node::S(&S105)],
};
static S105: Spline = Spline {
    coord: 3,
    loc: &[-0.01f32, 0.01f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(0.63f32), Node::C(0.3f32)],
};
static S106: Spline = Spline {
    coord: 2,
    loc: &[0.19999999f32, 0.44999996f32, 1.0f32],
    der: &[0.0f32, 0.0f32, 0.0f32],
    val: &[Node::C(0.0f32), Node::C(0.0f32), Node::S(&S107)],
};
static S107: Spline = Spline {
    coord: 3,
    loc: &[-0.01f32, 0.01f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(0.315f32), Node::C(0.15f32)],
};
static S108: Spline = Spline {
    coord: 2,
    loc: &[0.19999999f32, 0.44999996f32, 1.0f32],
    der: &[0.0f32, 0.0f32, 0.0f32],
    val: &[Node::C(0.0f32), Node::C(0.0f32), Node::S(&S109)],
};
static S109: Spline = Spline {
    coord: 3,
    loc: &[-0.01f32, 0.01f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(0.315f32), Node::C(0.15f32)],
};
static S110: Spline = Spline {
    coord: 1,
    loc: &[-1.0f32, -0.78f32, -0.5775f32, -0.375f32],
    der: &[0.0f32, 0.0f32, 0.0f32, 0.0f32],
    val: &[
        Node::S(&S111),
        Node::S(&S114),
        Node::S(&S116),
        Node::C(0.0f32),
    ],
};
static S111: Spline = Spline {
    coord: 2,
    loc: &[0.19999999f32, 0.44999996f32, 1.0f32],
    der: &[0.0f32, 0.0f32, 0.0f32],
    val: &[Node::C(0.0f32), Node::S(&S112), Node::S(&S113)],
};
static S112: Spline = Spline {
    coord: 3,
    loc: &[-0.01f32, 0.01f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(0.63f32), Node::C(0.3f32)],
};
static S113: Spline = Spline {
    coord: 3,
    loc: &[-0.01f32, 0.01f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(0.63f32), Node::C(0.3f32)],
};
static S114: Spline = Spline {
    coord: 2,
    loc: &[0.19999999f32, 0.44999996f32, 1.0f32],
    der: &[0.0f32, 0.0f32, 0.0f32],
    val: &[Node::C(0.0f32), Node::C(0.0f32), Node::S(&S115)],
};
static S115: Spline = Spline {
    coord: 3,
    loc: &[-0.01f32, 0.01f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(0.63f32), Node::C(0.3f32)],
};
static S116: Spline = Spline {
    coord: 2,
    loc: &[0.19999999f32, 0.44999996f32, 1.0f32],
    der: &[0.0f32, 0.0f32, 0.0f32],
    val: &[Node::C(0.0f32), Node::C(0.0f32), Node::S(&S117)],
};
static S117: Spline = Spline {
    coord: 3,
    loc: &[-0.01f32, 0.01f32],
    der: &[0.0f32, 0.0f32],
    val: &[Node::C(0.63f32), Node::C(0.3f32)],
};

pub const OFFSET_ADD: f32 = -0.5037500262260437f32;
pub static OFFSET: &Spline = &S0;
pub const FACTOR_ADD: f32 = 0.0f32;
pub static FACTOR: &Spline = &S53;
pub const JAGGEDNESS_ADD: f32 = 0.0f32;
pub static JAGGEDNESS: &Spline = &S102;

pub fn peaks_valleys(w: f32) -> f32 {
    -(((w.abs() - 2.0 / 3.0).abs()) - 1.0 / 3.0) * 3.0
}

fn eval_node(n: &Node, cv: &[f32; 4]) -> f32 {
    match n {
        Node::C(v) => *v,
        Node::S(s) => eval(s, cv),
    }
}

pub fn eval(sp: &Spline, cv: &[f32; 4]) -> f32 {
    let t = cv[sp.coord];
    let n = sp.loc.len();
    let i = sp.loc.partition_point(|&l| l <= t);

    if i == 0 {
        return eval_node(&sp.val[0], cv) + sp.der[0] * (t - sp.loc[0]);
    }
    if i == n {
        return eval_node(&sp.val[n - 1], cv) + sp.der[n - 1] * (t - sp.loc[n - 1]);
    }

    let k = i - 1;
    let (l0, l1) = (sp.loc[k], sp.loc[k + 1]);
    let f = (t - l0) / (l1 - l0);
    let v0 = eval_node(&sp.val[k], cv);
    let v1 = eval_node(&sp.val[k + 1], cv);
    let w = l1 - l0;
    let p0 = sp.der[k] * w - (v1 - v0);
    let p1 = -sp.der[k + 1] * w + (v1 - v0);

    v0 + (v1 - v0) * f + f * (1.0 - f) * (p0 + (p1 - p0) * f)
}
