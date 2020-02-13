
// ===================
// === BoundingBox ===
// ===================

/// Describes the rectangular convex hull of an object.
struct BoundingBox {
    float min_x;
    float max_x;
    float min_y;
    float max_y;
};

BoundingBox bounding_box (float min_x, float max_x, float min_y, float max_y) {
    return BoundingBox(min_x,max_x,min_y,max_y);
}

BoundingBox bounding_box (float w, float h) {
    return BoundingBox(-w,w,-h,h);
}

BoundingBox inverse (BoundingBox a) {
    return BoundingBox(0.0,0.0,0.0,0.0);
}

BoundingBox unify (BoundingBox a, BoundingBox b) {
    float min_x = min(a.min_x,b.min_x);
    float max_x = max(a.max_x,b.max_x);
    float min_y = min(a.min_y,b.min_y);
    float max_y = max(a.max_y,b.max_y);
    return BoundingBox(min_x,max_x,min_y,max_y);
}

BoundingBox intersection (BoundingBox a, BoundingBox b) {
    float min_x = max(a.min_x,b.min_x);
    float max_x = min(a.max_x,b.max_x);
    float min_y = max(a.min_y,b.min_y);
    float max_y = min(a.max_y,b.max_y);
    return BoundingBox(min_x,max_x,min_y,max_y);
}

BoundingBox difference (BoundingBox a, BoundingBox b) {
    return a;
}



// ===========
// === Sdf ===
// ===========

/// Signed distance field. Describes the distance to the nearest point of a shape.
/// Follow the link to learn more: https://en.wikipedia.org/wiki/Signed_distance_function .
struct Sdf {
    float distance;
};

Sdf sdf (float distance) {
    return Sdf(distance);
}

Sdf inverse (Sdf a) {
    return Sdf(-a.distance);
}

Sdf unify (Sdf a, Sdf b) {
    return Sdf(min(a.distance,b.distance));
}

Sdf intersection (Sdf a, Sdf b) {
    return Sdf(max(a.distance,b.distance));
}

Sdf difference (Sdf a, Sdf b) {
    return intersection(a,inverse(b));
}



// ================
// === BoundSdf ===
// ================

/// Bound SDF. Signed distance field with associated bounds. See documentation of `sdf` and `bbox`
/// to learn more.
struct BoundSdf {
    float       distance;
    BoundingBox bounds;
};


// === Getters ===

float distance (BoundSdf a) {
    return a.distance;
}

//BoundingBox bounds (BoundSdf a) {
//    return a.bounds;
//}

Sdf sdf (BoundSdf a) {
    return sdf(a.distance);
}


// === Smart Constructors ===

BoundSdf bound_sdf (float distance, BoundingBox bounds) {
    return BoundSdf(distance,bounds);
}

BoundSdf bound_sdf (Sdf sdf, BoundingBox bounds) {
    return bound_sdf(sdf.distance,bounds);
}


// === API ===

BoundSdf resample (BoundSdf a, float multiplier) {
    a.distance *= multiplier;
    return a;
}

BoundSdf pixel_snap (BoundSdf a) {
    a.distance = floor(a.distance) + 0.5;
    return a;
}

BoundSdf inverse (BoundSdf a) {
    return bound_sdf(inverse(sdf(a)),inverse(a.bounds));
}

BoundSdf unify (BoundSdf a, BoundSdf b) {
    return bound_sdf(unify(sdf(a),sdf(b)),unify(a.bounds,b.bounds));
}

BoundSdf difference (BoundSdf a, BoundSdf b) {
    return bound_sdf(difference(sdf(a),sdf(b)),difference(a.bounds,b.bounds));
}

BoundSdf intersection (BoundSdf a, BoundSdf b) {
    return bound_sdf(intersection(sdf(a),sdf(b)),intersection(a.bounds,b.bounds));
}



// ==========
// === Id ===
// ==========

struct Id {
    int value;
};

Id id(int value) {
    return Id(value);
}

Id new_id_layer (BoundSdf sdf, int i) {
    return Id((sdf.distance <= 0.0) ? i : 0);
}


// Premultiplied
struct Color {
    Rgba color;
};

Color premultiply(Rgba t) {
    float alpha = a(t);
    vec3 rgb    = t.raw.rgb * alpha;
    return Color(rgba(rgb,alpha));
}

Rgba unpremultiply(Color t) {
    float alpha = t.color.raw.a;
    vec3  rgb   = t.color.raw.rgb / alpha;
    return rgba(rgb,alpha);
}

// Implements glBlendFuncSeparate(GL_ONE,GL_ONE_MINUS_SRC_ALPHA,GL_ONE,GL_ONE_MINUS_SRC_ALPHA);
Color blend(Color bg, Color fg) {
    vec4 raw = fg.color.raw + (1.0 - fg.color.raw.a) * bg.color.raw;
    return Color(rgba(raw));
}



// =============
// === Shape ===
// =============

float render(BoundSdf sdf) {
    return clamp((-sdf.distance * input_pixel_ratio + 0.5) * input_zoom);
}

// Note: the color is premultiplied.
struct Shape {
    Id       id;
    BoundSdf sdf;
    Color    color;
    float    alpha;
};

Shape shape (Id id, BoundSdf bound_sdf, Rgba rgba) {
    float alpha = render(bound_sdf);
    rgba.raw.a *= alpha;
    Color color = premultiply(rgba);
    return Shape(id,bound_sdf,color,alpha);
}

Shape shape (Id id, BoundSdf bound_sdf, Color color) {
    float alpha = render(bound_sdf);
    return Shape(id,bound_sdf,color,alpha);
}

Shape resample (Shape s, float multiplier) {
    Id       id    = s.id;
    BoundSdf sdf   = resample(s.sdf,multiplier);
    Rgba     color = unpremultiply(s.color);
    color.raw.a /= s.alpha;
    return shape(id,sdf,color);
}

Shape pixel_snap (Shape s) {
    Id       id    = s.id;
    BoundSdf sdf   = pixel_snap(s.sdf);
    Rgba     color = unpremultiply(s.color);
    color.raw.a /= s.alpha;
    return shape(id,sdf,color);
}

Shape inverse (Shape s1) {
    return shape(s1.id,inverse(s1.sdf),s1.color);
}

Shape unify (Shape s1, Shape s2) {
    return shape(s1.id,unify(s1.sdf,s2.sdf),blend(s1.color,s2.color));
}

Shape difference (Shape s1, Shape s2) {
    return shape(s1.id,difference(s1.sdf,s2.sdf),s1.color);
}

Shape intersection (Shape s1, Shape s2) {
    return shape(s1.id,intersection(s1.sdf,s2.sdf),blend(s1.color,s2.color));
}

Shape set_color(Shape shape, Rgba t) {
    t.raw.a *= shape.alpha;
    Color color = premultiply(t);
    shape.color = color;
    return shape;
}


// ===========
// === Env ===
// ===========

struct Env {
    int test;
};



///////////////////////
////// Transform //////
///////////////////////

vec2 translate (vec2 p, vec2 t) {
    return p - t;
}

vec2 rotate (vec2 p, float angle) {
    return p*cos(-angle) + vec2(p.y,-p.x)*sin(-angle);
}

vec2 scale (vec2 p, float value) {
    return p/value;
}

vec2 cartesian2polar (vec2 p) {
  return vec2(length(p), atan(p.y, p.x));
}
