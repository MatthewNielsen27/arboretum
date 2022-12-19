pub mod prelude;
pub mod point_quadtree;

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use crate::spatial::quadtree::prelude::*;
    use crate::spatial::quadtree::point_quadtree::*;

    #[test]
    fn test_BBox2D() {
        let bbox = BBox2D {
            min: Vec2::from([-10.0, -10.0]),
            max: Vec2::from([10.0, 10.0])
        };

        assert!(bbox.contains(&Vec2::default()));
        assert!(!bbox.contains(&Vec2::from([20.0, -100.0])));
        assert!(!bbox.contains(&Vec2::from([-80.0, 10.0])));

        let [sw, se, ne, nw] = bbox.subdivide(&bbox.mid());
        assert!(!sw.contains(&Vec2::default()));
        assert!(!se.contains(&Vec2::default()));
        assert!(ne.contains(&Vec2::default()));
        assert!(!nw.contains(&Vec2::default()));

        assert!(bbox.intersects(&sw));
        assert!(bbox.intersects(&se));
        assert!(bbox.intersects(&ne));
        assert!(bbox.intersects(&nw));
    }

    #[test]
    fn test_PointQuadtree() {
        let bbox = BBox2D {
            min: Vec2::from([-10.0, -10.0]),
            max: Vec2::from([10.0, 10.0])
        };

        let p1 = Vec2::from([0.0, 0.0]);
        let p2 = Vec2::from([1.0, 4.0]);
        let p3 = Vec2::from([-2.0, 3.0]);

        let mut tree = PointQuadtree::<i32>::new(&bbox);
        assert_eq!(tree.len(), 0);

        assert!(tree.insert(&p1, 12));
        assert_eq!(tree.len(), 1);

        assert!(!tree.insert(&p1, 14));
        assert_eq!(tree.len(), 1);

        // Now let's try to find 'p1'
        let (_, item) = tree.find(&p1).unwrap();
        assert_eq!(item, 12);

        assert!(tree.insert(&p2, -1));
        assert!(tree.insert(&p3, 4));

        // Now let's try a query points in a region
        let region = BBox2D {
            min: Vec2::from([0.0, 0.0]),
            max: Vec2::from([10.0, 10.0])
        };
        let items = tree.find_within(&region);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].1, 12);
        assert_eq!(items[1].1, -1);
    }
}
