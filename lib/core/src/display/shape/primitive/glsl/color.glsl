// =================================================================================================
// === Colorspaces Definition ======================================================================
// =================================================================================================

/// Helper for colors definition.
/// For the given input of `DEF_COLOR(Rgb,Rgba,rgb,rgba,r,g,b)`, it defines:
///   - A Rgb  struct which contains `vec3 raw` field for each component.
///   - A Rgba struct which contains `vec4 raw` field for each component.
///   - A rich set of smart constructors for each of the types.
///   - A rich set of simple conversions between them.

#define DEF_COLOR(type_name3,type_name4,name3,name4,t1,t2,t3) \
                                                              \
/* ============================= */                           \
/* === Non Transparent Color === */                           \
/* ============================= */                           \
                                                              \
/* === Definition === */                                      \
                                                              \
struct type_name3 {                                           \
   vec3 raw;                                                  \
};                                                            \
                                                              \
                                                              \
/* === Getters === */                                         \
                                                              \
float t1 (type_name3 color) { return color.raw.x; }           \
float t2 (type_name3 color) { return color.raw.y; }           \
float t3 (type_name3 color) { return color.raw.z; }           \
                                                              \
                                                              \
/* === Constructors === */                                    \
                                                              \
type_name3 name3(type_name3 identity) {                       \
    return identity;                                          \
}                                                             \
                                                              \
type_name3 name3(vec3 raw) {                                  \
    return type_name3(raw);                                   \
}                                                             \
                                                              \
type_name3 name3(float t1, float t2, float t3) {              \
    return name3(vec3(t1,t2,t3));                             \
}                                                             \
                                                              \
                                                              \
                                                              \
/* ========================= */                               \
/* === Transparent Color === */                               \
/* ========================= */                               \
                                                              \
/* === Definition === */                                      \
                                                              \
struct type_name4 {                                           \
   vec4 raw;                                                  \
};                                                            \
                                                              \
                                                              \
/* === Getters === */                                         \
                                                              \
float t1 (type_name4 name4) { return name4.raw.x; }           \
float t2 (type_name4 name4) { return name4.raw.y; }           \
float t3 (type_name4 name4) { return name4.raw.z; }           \
float a  (type_name4 name4) { return name4.raw.a; }           \
                                                              \
                                                              \
/* === Constructors === */                                    \
                                                              \
type_name4 name4 (type_name4 identity) {                      \
    return identity;                                          \
}                                                             \
                                                              \
type_name4 name4 (vec4 raw) {                                 \
    return type_name4(raw);                                   \
}                                                             \
                                                              \
type_name4 name4 (vec3 raw) {                                 \
    return name4(vec4(raw,1.0));                              \
}                                                             \
                                                              \
type_name4 name4 (vec3 raw, float a) {                        \
    return name4(vec4(raw,a));                                \
}                                                             \
                                                              \
type_name4 name4 (type_name3 name3) {                         \
    return name4(name3.raw);                                  \
}                                                             \
                                                              \
type_name4 name4 (type_name3 name3, float a) {                \
    return name4(name3.raw,a);                                \
}                                                             \
                                                              \
type_name4 name4 (float t1, float t2, float t3) {             \
    return name4(vec3(t1,t2,t3));                             \
}                                                             \
                                                              \
type_name4 name4 (float t1, float t2, float t3, float a) {    \
    return name4(vec4(t1,t2,t3,a));                           \
}                                                             \
                                                              \
                                                              \
/* === Conversions === */                                     \
                                                              \
type_name3 name3 (type_name4 a) {                             \
    return name3(a.raw.xyz);                                  \
}


DEF_COLOR(Srgb , SRgba , srgb , srgba , r , g ,b)
DEF_COLOR(Rgb  , Rgba  , rgb  , rgba  , r , g ,b)
DEF_COLOR(HSV  , HSVA  , hsv  , hsva  , h , s ,v)
DEF_COLOR(Lch  , Lcha  , lch  , lcha  , l , c ,h)



// =================================================================================================
// === Colorpsace Conversion =======================================================================
// =================================================================================================

#define DEF_TRANSITIVE_CONVERSIONS_1_WAY(                                                 \
a_type_name3,a_type_name4,a_name3,a_name4,b_type_name3,b_type_name4,b_name3,b_name4) \
                                                                                     \
a_type_name3 a_name3(b_type_name4 b_name4) {                                         \
    return a_name3(b_name3(b_name4));                                                \
}                                                                                    \
                                                                                     \
a_type_name4 a_name4(b_type_name4 b_name4) {                                         \
    return a_name4(a_name3(b_name3(b_name4)),a(b_name4));                            \
}                                                                                    \
                                                                                     \
a_type_name4 a_name4(b_type_name3 b_name3) {                                         \
    return a_name4(a_name3(b_name3));                                                \
}

#define DEF_TRANSITIVE_CONVERSIONS(                                                       \
a_type_name3,a_type_name4,a_name3,a_name4,b_type_name3,b_type_name4,b_name3,b_name4) \
DEF_TRANSITIVE_CONVERSIONS_1_WAY(                                                         \
a_type_name3,a_type_name4,a_name3,a_name4,b_type_name3,b_type_name4,b_name3,b_name4) \
DEF_TRANSITIVE_CONVERSIONS_1_WAY(                                                         \
b_type_name3,b_type_name4,b_name3,b_name4,a_type_name3,a_type_name4,a_name3,a_name4)



// ====================
// === Rgb <-> Srgb ===
// ====================

const float SRGB_ALPHA = 0.055;

float channel_rgb_to_srgb(float channel) {
    if(channel <= 0.0031308)
    return 12.92 * channel;
    else
    return (1.0 + SRGB_ALPHA) * pow(channel, 1.0/2.4) - SRGB_ALPHA;
}

float channel_srgb_to_rgb(float channel) {
    if (channel <= 0.04045)
    return channel / 12.92;
    else
    return pow((channel + SRGB_ALPHA) / (1.0 + SRGB_ALPHA), 2.4);
}

Srgb srgb(Rgb rgb) {
    float r = channel_rgb_to_srgb(r(rgb));
    float g = channel_rgb_to_srgb(g(rgb));
    float b = channel_rgb_to_srgb(b(rgb));
    return srgb(r,g,b);
}

Rgb rgb(Srgb srgb) {
    float r = channel_srgb_to_rgb(r(srgb));
    float g = channel_srgb_to_rgb(g(srgb));
    float b = channel_srgb_to_rgb(b(srgb));
    return rgb(r,g,b);
}

DEF_TRANSITIVE_CONVERSIONS(Srgb,SRgba,srgb,srgba,Rgb,Rgba,rgb,rgba)




// ====================
// === Srgb <-> Lch ===
// ====================

vec3 lch_rgb_weights = vec3(1.0,1.0,1.0);
float xyzF(float t){ return mix(pow(t,1./3.), 7.787037 * t + 0.139731  , step(t,0.00885645)); }
float xyzR(float t){ return mix(t*t*t       , 0.1284185*(t - 0.139731) , step(t,0.20689655)); }
Lch lch(Srgb rgb) {
    vec3 c = rgb.raw;
    c *= mat3
        ( 0.4124, 0.3576, 0.1805
        , 0.2126, 0.7152, 0.0722
        , 0.0193, 0.1192, 0.9505 );
    c.x = xyzF(c.x/lch_rgb_weights.x);
    c.y = xyzF(c.y/lch_rgb_weights.y);
    c.z = xyzF(c.z/lch_rgb_weights.z);
    vec3 lab = vec3(max(0.,116.0*c.y - 16.0), 500.0*(c.x - c.y), 200.0*(c.y - c.z));
    return lch(lab.x, length(vec2(lab.y,lab.z)), atan(lab.z, lab.y));
}

Srgb srgb (Lch lch) {
    vec3 c = lch.raw;
    c = vec3(c.x, cos(c.z) * c.y, sin(c.z) * c.y);
    float lg = 1./116.*(c.x + 16.);
    float x  = lch_rgb_weights.x*xyzR(lg + 0.002*c.y);
    float y  = lch_rgb_weights.y*xyzR(lg);
    float z  = lch_rgb_weights.z*xyzR(lg - 0.005*c.z);
    vec3 xyz = vec3(x,y,z);
    vec3 raw = xyz * mat3
        (  3.2406, -1.5372, -0.4986
        , -0.9689,  1.8758,  0.0415
        ,  0.0557, -0.2040,  1.0570 );
    return srgb(raw);
}

DEF_TRANSITIVE_CONVERSIONS(Srgb,SRgba,srgb,srgba,Lch,Lcha,lch,lcha)



// ====================
// === Srgb <-> HSV ===
// ====================

HSV hsv(Srgb rgb) {
    vec3  c = rgb.raw;
    vec4  K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4  p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4  q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    float h = abs(q.z + (q.w - q.y) / (6.0 * d + e));
    float s = d / (q.x + e);
    float v = q.x;
    return hsv(h,s,v);
}

Srgb srgb(HSV hsv) {
    vec3 c = hsv.raw;
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return srgb(c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y));
}

DEF_TRANSITIVE_CONVERSIONS(Srgb,SRgba,srgb,srgba,HSV,HSVA,hsv,hsva)


// TODO: we will use it for color mixing. Should be enabled / removed in next PR.

////cheaply lerp around a circle
//float lerpAng(in float a, in float b, in float x)
//{
//    float ang = mod(mod((a-b), TAU) + PI*3., TAU)-PI;
//    return ang*x+b;
//}
//
////Linear interpolation between two colors in Lch space
//vec3 lerpLch(in vec3 a, in vec3 b, in float x)
//{
//    float hue = lerpAng(a.z, b.z, x);
//    return vec3(mix(b.xy, a.xy, x), hue);
//}



// =================================================================================================
// === Mixing ======================================================================================
// =================================================================================================

Rgb mix(Rgb color1, Rgb color2, float t) {
    return rgb(mix(color1.raw,color2.raw,t));
}

Rgba mix(Rgba color1, Rgba color2, float t) {
    return rgba(mix(color1.raw,color2.raw,t));
}




// ================
// === Gradient ===
// ================

struct RgbGradientControlPoint {
    float offset;
    Rgb   color;
};

RgbGradientControlPoint gradient_control_point(float offset, Rgb color) {
    return RgbGradientControlPoint(offset,color);
}




struct RgbGradient2 {
    RgbGradientControlPoint control_point1;
    RgbGradientControlPoint control_point2;
};

RgbGradient2 gradient
(RgbGradientControlPoint control_point1, RgbGradientControlPoint control_point2) {
    return RgbGradient2(control_point1,control_point2);
}


Rgb samplex(float offset, RgbGradient2 gradient) {
    float span = gradient.control_point2.offset - gradient.control_point1.offset;
    float t    = (offset - gradient.control_point1.offset) / span;
    float t2   = clamp(t);
    return rgb(mix(gradient.control_point1.color.raw,gradient.control_point2.color.raw,t2));
}
