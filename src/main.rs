use std::error::Error;

const SIGN_BIT: u32 = 1u32 << (i32::BITS as u32 - 1);

trait Interleavable {
    type InterleaveOutput;
    fn interleave(self, other: Self) -> Self::InterleaveOutput; 
}

impl Interleavable for u8 {
    type InterleaveOutput = u16;
    fn interleave(self, other: Self) -> Self::InterleaveOutput {
        let x = self as Self::InterleaveOutput;
        let x = (x | (x << 4)) & 0x0F0F;
        let x = (x | (x << 2)) & 0x3333;
        let x = (x | (x << 1)) & 0x5555;

        let y = other as Self::InterleaveOutput;
        let y = (y | (y << 4)) & 0x0F0F;
        let y = (y | (y << 2)) & 0x3333;
        let y = (y | (y << 1)) & 0x5555;
        let y = y << 1;
        x | y
    }
}

impl Interleavable for u16 {
    type InterleaveOutput = u32;
    fn interleave(self, other: Self) -> Self::InterleaveOutput {
        let x = self as Self::InterleaveOutput;
        let x = (x | (x << 8)) & 0x00FF00FF;
        let x = (x | (x << 4)) & 0x0F0F0F0F;
        let x = (x | (x << 2)) & 0x33333333;
        let x = (x | (x << 1)) & 0x55555555;

        let y = other as Self::InterleaveOutput;
        let y = (y | (y << 8)) & 0x00FF00FF;
        let y = (y | (y << 4)) & 0x0F0F0F0F;
        let y = (y | (y << 2)) & 0x33333333;
        let y = (y | (y << 1)) & 0x55555555;
        let y = y << 1;
        x | y
    }
}

impl Interleavable for u32 {
    type InterleaveOutput = u64;
    fn interleave(self, other: Self) -> Self::InterleaveOutput {
        let x = self as Self::InterleaveOutput;
        let x = (x | (x << 16)) & 0x0000FFFF0000FFFF;
        let x = (x | (x << 8)) & 0x00FF00FF00FF00FF;
        let x = (x | (x << 4)) & 0x0F0F0F0F0F0F0F0F;
        let x = (x | (x << 2)) & 0x3333333333333333;
        let x = (x | (x << 1)) & 0x5555555555555555;

        let y = other as Self::InterleaveOutput;
        let y = (y | (y << 16)) & 0x0000FFFF0000FFFF;
        let y = (y | (y << 8)) & 0x00FF00FF00FF00FF;
        let y = (y | (y << 4)) & 0x0F0F0F0F0F0F0F0F;
        let y = (y | (y << 2)) & 0x3333333333333333;
        let y = (y | (y << 1)) & 0x5555555555555555;
        let y = y << 1;
        x | y
    }
}

impl Interleavable for u64 {
    type InterleaveOutput = u128;
    fn interleave(self, other: Self) -> Self::InterleaveOutput {
        let x = self as Self::InterleaveOutput;
        let x = (x | (x << 32)) & 0x00000000FFFFFFFF00000000FFFFFFFF;
        let x = (x | (x << 16)) & 0x0000FFFF0000FFFF0000FFFF0000FFFF;
        let x = (x | (x << 8)) & 0x00FF00FF00FF00FF00FF00FF00FF00FF;
        let x = (x | (x << 4)) & 0x0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F;
        let x = (x | (x << 2)) & 0x33333333333333333333333333333333;
        let x = (x | (x << 1)) & 0x55555555555555555555555555555555;

        let y = other as Self::InterleaveOutput;
        let y = (y | (y << 32)) & 0x00000000FFFFFFFF00000000FFFFFFFF;
        let y = (y | (y << 16)) & 0x0000FFFF0000FFFF0000FFFF0000FFFF;
        let y = (y | (y << 8)) & 0x00FF00FF00FF00FF00FF00FF00FF00FF;
        let y = (y | (y << 4)) & 0x0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F;
        let y = (y | (y << 2)) & 0x33333333333333333333333333333333;
        let y = (y | (y << 1)) & 0x55555555555555555555555555555555;
        let y = y << 1;
        x | y
    }
}

enum TreeNode<Data> {
    Leaf(Data),
    Branch(Vec<TreeNode<Data>>),
    Empty,
}

pub type OctreeResult<T> = Result<T, Box<dyn Error>>;

pub struct Octree<Data> {
    root_node: TreeNode<Data>,
    depth: u32,
}

impl<Data> Octree<Data> {
    /// [depth], where 2^depth = density of data (how many items across the side of the tree at the
    /// max depth)
    pub fn new(depth: u32) -> Self {
        Self { root_node: TreeNode::Empty, depth }
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> Option<&Data> {
        self.inner_get_iterative(Self::encode_position(x, y, z, self.depth))
    }

    pub fn insert(&mut self, x: i32, y: i32, z: i32, data: Data) -> OctreeResult<()> {
        self.inner_insert_iterative(Self::encode_position(x, y, z, self.depth), 0, data)
    }

    fn encode_position(x: i32, y: i32, z: i32, depth: u32) -> u128 {
        fn q(a: i32, shft: u32) -> u32 {
            (((a as u32) << shft) ^ SIGN_BIT).reverse_bits()
        }
        let shft = u32::BITS - depth;
        let sx = q(x, shft);
        let sy = q(y, shft);
        let sz = q(z, shft);
        u64::interleave(u32::interleave(sx, sy), u32::interleave(sz, 0))
    }

    fn inner_get_iterative(&self, mut interweaved_lookup: u128) -> Option<&Data> {
        let mut parent_node = &self.root_node;
        loop {
            match parent_node {
                TreeNode::Leaf(data) => { return Some(&data) },
                TreeNode::Branch(child_branches) => {
                    // get the 0 -> 7 value representing the child octant
                    let octant = &child_branches[(interweaved_lookup & 15) as usize];
                        // shift the octant's position out of the lookup value
                        interweaved_lookup = interweaved_lookup >> 4;
                        parent_node = octant;
                },
                TreeNode::Empty => { return None; }
            }
        }
    }

    fn inner_insert_iterative(&mut self, mut interweaved_lookup: u128, mut current_depth: u32, new_data: Data) -> OctreeResult<()> {
        let mut current_node = &mut self.root_node;
        loop {
            match current_node {
                TreeNode::Branch(child_branches) => {
                    // get the 0 -> 7 value representing the child octant
                    let octant = &mut child_branches[(interweaved_lookup & 15) as usize];
                    // dont let us recurse too far down
                    if current_depth >= self.depth { 
                        return Err("Surpassed depth of tree with branches?".into()); 
                    }
                    // shift the octant's position out of the lookup value
                    interweaved_lookup = interweaved_lookup >> 4;
                    current_node = octant;
                    current_depth += 1;
                },
                TreeNode::Leaf(data) => {
                    if current_depth < self.depth { 
                        return Err(format!("Leaf Node before full depth: {}", current_depth).into()); 
                    }
                    *data = new_data; 
                    return Ok(());
                },
                TreeNode::Empty => {
                    if current_depth < self.depth {
                        *current_node = TreeNode::Branch(vec![TreeNode::Empty, TreeNode::Empty, TreeNode::Empty, TreeNode::Empty, TreeNode::Empty, TreeNode::Empty, TreeNode::Empty, TreeNode::Empty]);       
                    } else {
                        // Create new Leaf
                        *current_node = TreeNode::Leaf(new_data);
                        return Ok(());
                    }
                }
            }
        }
    }

}

fn main() {
    const WORLD_DEPTH: u32 = 8;
    fn f_pos(x: i32, y: i32, z: i32) -> u128 {
        x as u128 | ((y as u128) << 32) | ((z as u128) << 96)
    }
    let world_width = 2i32.pow(WORLD_DEPTH);
    let world_max = world_width >> 1;
    let world_min = -world_max;

    println!("Creating Tree Object");
    let mut octree: Octree<u128> = Octree::new(WORLD_DEPTH);

    println!("Generating Tree");
    // Generate tree
    for y in world_min..world_max {
        for z in world_min..world_max {
            for x in world_min..world_max {
                octree.insert(x, y, z, f_pos(x, y, z)).unwrap();
            }
        }
    }

    println!("Testing Tree");
    // Test Tree
    for y in world_min..world_max {
        for z in world_min..world_max {
            for x in world_min..world_max {
                let stored = octree.get(x, y, z);
                assert_eq!(stored.cloned(), Some(f_pos(x, y, z)));
            }
        }
    }
    println!("Tree successful!");
}
