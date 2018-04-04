
struct Point {
    x: f32,
    y: f32,
}

struct Rectangle {
    p1: Point,
    p2: Point,
}


fn rect_area(r: Rectangle) -> f32 {
    let Rectangle {
        p1: Point { x: x1, y: y1 },
        p2: Point { x: x2, y: y2 },
    } = r;

    return (x2-x1) * (y2-y1);
}

fn square(p: Point, f: f32) -> Rectangle {
    
}

fn main() {

    let r = Rectangle {
        p1: Point {x: 1.0, y: 1.0},
        p2: Point {x: 2.0, y: 5.0},
    };

    println!("area: {}", rect_area(r))

}
