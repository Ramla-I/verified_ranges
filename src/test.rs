mod range_inclusive {
    use crate::{
        addr::*,
        unit::*,
        range_inclusive::*,
        range::*
    };

    #[test]
    fn greater_end() {
        let range = RangeInclusive::new(Frame{ number: 0} , Frame{ number: 1 });
        assert!(!range.is_empty());
        for r in range.iter() {
            println!("{:?}", r);
        }
        println!("original range: {:?} \n", range);

        let range = RangeInclusive::new(Frame{ number: 10} , Frame{ number: 17 });
        assert!(!range.is_empty());
        for r in range.iter() {
            println!("{:?}", r);
        }
        println!("original range: {:?} \n", range);
        
        let range = RangeInclusive::new(Frame{ number: 3} , Frame{ number: 22 });
        assert!(!range.is_empty());
        for r in range.iter() {
            println!("{:?}", r);
        }
        println!("original range: {:?} \n", range);
        
        let range = RangeInclusive::new(Frame{ number: 597} , Frame{ number: 782 });
        assert!(!range.is_empty());
        for r in range.iter() {
            println!("{:?}", r);
        }
        println!("original range: {:?} \n", range);
    }

    #[test]
    fn equal_start_end() {
        let range = RangeInclusive::new(Frame{ number: 0} , Frame{ number: 0});
        assert!(!range.is_empty());
        for r in range.iter() {
            println!("{:?}", r);
        }
        println!("original range: {:?} \n", range);
        
        let range = RangeInclusive::new(Frame{ number: 597} , Frame{ number: 597});
        assert!(!range.is_empty());
        for r in range.iter() {
            println!("{:?}", r);
        }
        println!("original range: {:?} \n", range);
    }

    #[test]
    fn greater_start() {
        let range = RangeInclusive::new(Frame{ number: 782} , Frame{ number: 597 });
        assert!(range.is_empty());
        for r in range.iter() {
            println!("{:?}", r);
        }
        println!("original range: {:?} \n", range);
        
        let range = RangeInclusive::new(Frame{ number: 1} , Frame{ number: 0 });
        assert!(range.is_empty());
        for r in range.iter() {
            println!("{:?}", r);
        }
        println!("original range: {:?} \n", range);
    }
}

mod range {
    use crate::{
        addr::*,
        unit::*,
        range_inclusive::*,
        range::*
    };

    #[test]
    fn test_contains() {
        let fr = FrameRange::new(Frame{ number: 1 }, Frame{ number: 5 });
        assert!(fr.contains(&Frame{ number: 3 }));
        assert!(fr.contains(&Frame{ number: 1 }));
        assert!(fr.contains(&Frame{ number: 5 }));
        assert!(!fr.contains(&Frame{ number: 0 }));
        assert!(!fr.contains(&Frame{ number: 6 }));

    }

    #[test]
    fn test_iterator() {
        let fr = FrameRange::new(Frame{ number: 1 }, Frame{ number: 5 });
        assert!(!fr.is_empty());
        for r in fr.iter() {
            println!("{:?}", r);
        }
        println!("original range: {:?} \n", fr);

        let fr = FrameRange::new(Frame{ number: 1 }, Frame{ number: 0 });
        assert!(fr.is_empty());
        for r in fr.iter() {
            println!("{:?}", r);
        }
        println!("original range: {:?} \n", fr);
    }


}