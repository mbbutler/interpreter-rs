#[macro_export]
macro_rules! binary_op {
    ( $self:expr, $op:tt ) => {
        {
            let b = $self.pop()?;
            let a = $self.pop()?;
            $self.push(a $op b)?;
        }
    };
}
