//! Pure game rules (no engine types), so they can be unit tested.

pub const N: usize = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toggle {
    Ignored,
    Rejected,
    Selected,
    Deselected,
}

#[derive(Clone, Debug)]
pub struct Board {
    pub open: [bool; N],
    pub selected: [bool; N],
    pub roll_total: u32,
    pub selected_sum: u32,
}

impl Default for Board {
    fn default() -> Self {
        Self { open: [true; N], selected: [false; N], roll_total: 0, selected_sum: 0 }
    }
}

impl Board {
    pub fn reset(&mut self) {
        *self = Board::default();
    }

    pub fn open_sum(&self) -> u32 {
        (0..N).filter(|&i| self.open[i]).map(|i| i as u32 + 1).sum()
    }

    /// Only one die is rolled once 7, 8 and 9 are all shut.
    pub fn one_die(&self) -> bool {
        !self.open[6] && !self.open[7] && !self.open[8]
    }

    /// Is there any subset of the open tiles that adds up to the current roll?
    pub fn has_move(&self) -> bool {
        let open: Vec<u32> = (0..N).filter(|&i| self.open[i]).map(|i| i as u32 + 1).collect();
        (1u32..(1 << open.len())).any(|mask| {
            let s: u32 = open
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, n)| *n)
                .sum();
            s == self.roll_total
        })
    }

    pub fn toggle(&mut self, idx: usize) -> Toggle {
        if idx >= N || !self.open[idx] {
            return Toggle::Ignored;
        }
        let n = idx as u32 + 1;
        if self.selected[idx] {
            self.selected[idx] = false;
            self.selected_sum -= n;
            Toggle::Deselected
        } else if self.selected_sum + n > self.roll_total {
            Toggle::Rejected
        } else {
            self.selected[idx] = true;
            self.selected_sum += n;
            Toggle::Selected
        }
    }

    pub fn matched(&self) -> bool {
        self.roll_total > 0 && self.selected_sum == self.roll_total
    }

    pub fn clear_selection(&mut self) {
        self.selected = [false; N];
        self.selected_sum = 0;
    }

    /// Shut the selected tiles. Returns their indices.
    pub fn close_selected(&mut self) -> Vec<usize> {
        let closed: Vec<usize> = (0..N).filter(|&i| self.selected[i]).collect();
        for &i in &closed {
            self.open[i] = false;
        }
        self.clear_selection();
        closed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_open_every_roll_possible() {
        let mut b = Board::default();
        for r in 2..=12 {
            b.roll_total = r;
            assert!(b.has_move(), "roll {r}");
        }
    }

    #[test]
    fn no_move_when_only_one_is_open_and_roll_two() {
        let mut b = Board::default();
        b.open = [false; N];
        b.open[0] = true;
        b.roll_total = 2;
        assert!(!b.has_move());
    }

    #[test]
    fn one_die_rule() {
        let mut b = Board::default();
        assert!(!b.one_die());
        b.open[6] = false;
        b.open[7] = false;
        assert!(!b.one_die());
        b.open[8] = false;
        assert!(b.one_die());
    }

    #[test]
    fn toggle_rejects_overshoot_and_matches() {
        let mut b = Board::default();
        b.roll_total = 5;
        assert_eq!(b.toggle(5), Toggle::Rejected); // tile 6 > 5
        assert_eq!(b.toggle(1), Toggle::Selected); // 2
        assert!(!b.matched());
        assert_eq!(b.toggle(2), Toggle::Selected); // 3
        assert!(b.matched());
        assert_eq!(b.close_selected(), vec![1, 2]);
        assert_eq!(b.open_sum(), 45 - 5);
    }

    #[test]
    fn deselect_restores_sum() {
        let mut b = Board::default();
        b.roll_total = 9;
        b.toggle(3);
        assert_eq!(b.selected_sum, 4);
        assert_eq!(b.toggle(3), Toggle::Deselected);
        assert_eq!(b.selected_sum, 0);
    }
}
