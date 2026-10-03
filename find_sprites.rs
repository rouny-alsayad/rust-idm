use image::GenericImageView;
use std::collections::VecDeque;

fn main() {
    let img = image::open("src/icon/icons.png").unwrap().to_rgba8();
    let (w, h) = img.dimensions();
    let mut visited = vec![false; (w * h) as usize];
    let mut rects = vec![];

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if !visited[idx] && img.get_pixel(x, y)[3] > 0 {
                let mut q = VecDeque::new();
                q.push_back((x, y));
                visited[idx] = true;

                let mut min_x = x;
                let mut max_x = x;
                let mut min_y = y;
                let mut max_y = y;

                while let Some((cx, cy)) = q.pop_front() {
                    min_x = min_x.min(cx);
                    max_x = max_x.max(cx);
                    min_y = min_y.min(cy);
                    max_y = max_y.max(cy);

                    let dx = [0, 1, 0, -1, 1, 1, -1, -1];
                    let dy = [-1, 0, 1, 0, -1, 1, -1, 1];
                    for i in 0..8 {
                        let nx = cx as i32 + dx[i];
                        let ny = cy as i32 + dy[i];
                        if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                            let nx = nx as u32;
                            let ny = ny as u32;
                            let nidx = (ny * w + nx) as usize;
                            if !visited[nidx] && img.get_pixel(nx, ny)[3] > 0 {
                                visited[nidx] = true;
                                q.push_back((nx, ny));
                            }
                        }
                    }
                }
                rects.push((min_x, min_y, max_x - min_x + 1, max_y - min_y + 1));
            }
        }
    }
    
    loop {
        let mut merged = false;
        for i in 0..rects.len() {
            for j in (i+1)..rects.len() {
                let (x1, y1, w1, h1) = rects[i];
                let (x2, y2, w2, h2) = rects[j];
                
                let overlap = !(x1 + w1 + 5 < x2 || x2 + w2 + 5 < x1 || y1 + h1 + 5 < y2 || y2 + h2 + 5 < y1);
                if overlap {
                    let mx = x1.min(x2);
                    let my = y1.min(y2);
                    let mx2 = (x1 + w1).max(x2 + w2);
                    let my2 = (y1 + h1).max(y2 + h2);
                    rects[i] = (mx, my, mx2 - mx, my2 - my);
                    rects.remove(j);
                    merged = true;
                    break;
                }
            }
            if merged { break; }
        }
        if !merged { break; }
    }

    rects.sort_by_key(|a| (a.1 / 100) * 10000 + a.0);

    for (i, r) in rects.iter().enumerate() {
        if r.2 > 20 && r.3 > 20 {
            println!("Icon {}: x={}, y={}, w={}, h={}", i, r.0, r.1, r.2, r.3);
        }
    }
}