use approx::relative_eq;

pub type Points = Measurement<Scaled>;
pub type Pixels = Measurement<Raw>;

#[derive(Clone, Copy, PartialOrd, PartialEq, Debug, Default)]
pub struct Scaled(pub f32);
#[derive(Clone, Copy, PartialOrd, PartialEq, Debug, Default)]
pub struct Raw(pub f32);

#[derive(Clone, Copy, PartialOrd, PartialEq, Debug, Default)]
pub struct Measurement<T> {
    value: T,
}

impl<T> Measurement<T>
where
    T: ScreenMeasurement + PartialOrd + Copy,
{
    pub fn max(&self, other: Self) -> Self {
        if relative_eq!(self.value.to_f32(), other.value.to_f32()) || self < &other {
            other
        } else {
            *self
        }
    }

    pub fn min(&self, other: Self) -> Self {
        if relative_eq!(self.value.to_f32(), other.value.to_f32()) || self > &other {
            other
        } else {
            *self
        }
    }
}

impl<T> ScreenMeasurement for Measurement<T>
where
    T: ScreenMeasurement,
{
    fn from_f32(value: f32) -> Self {
        Self {
            value: T::from_f32(value),
        }
    }

    fn to_points(&self, effective_scale: f32) -> Points {
        self.value.to_points(effective_scale)
    }

    fn to_pixels(&self, effective_scale: f32) -> Pixels {
        self.value.to_pixels(effective_scale)
    }

    fn to_f32(&self) -> f32 {
        self.value.to_f32()
    }
}

impl<T> From<T> for Measurement<T>
where
    T: ScreenMeasurement,
{
    fn from(value: T) -> Self {
        Self { value }
    }
}

impl<T> From<f32> for Measurement<T>
where
    T: ScreenMeasurement,
{
    fn from(value: f32) -> Self {
        Self {
            value: T::from_f32(value),
        }
    }
}

impl<T> Into<f32> for Measurement<T>
where
    T: ScreenMeasurement,
{
    fn into(self) -> f32 {
        self.value.to_f32()
    }
}

pub trait ScreenMeasurement {
    fn from_f32(value: f32) -> Self;
    fn to_f32(&self) -> f32;

    fn to_pixels(&self, effective_scale: f32) -> Pixels;
    fn to_points(&self, effective_scale: f32) -> Points;
}

impl ScreenMeasurement for Scaled {
    fn from_f32(value: f32) -> Self {
        Self(value)
    }
    fn to_f32(&self) -> f32 {
        self.0
    }

    fn to_pixels(&self, effective_scale: f32) -> Pixels {
        Pixels::from(Raw(self.0 * effective_scale))
    }

    fn to_points(&self, _effective_scale: f32) -> Points {
        Points::from(*self)
    }
}

impl ScreenMeasurement for Raw {
    fn to_pixels(&self, _effective_scale: f32) -> Pixels {
        Pixels::from(*self)
    }

    fn to_points(&self, effective_scale: f32) -> Points {
        Points::from(Scaled(self.0 as f32 / effective_scale))
    }

    fn to_f32(&self) -> f32 {
        self.0 as f32
    }

    fn from_f32(value: f32) -> Self {
        Self(value)
    }
}

impl<T> std::ops::Mul<Self> for Measurement<T>
where
    T: ScreenMeasurement,
{
    type Output = Self;

    fn mul(self, s: Self) -> Self {
        Self {
            value: T::from_f32(self.value.to_f32() * s.value.to_f32()),
        }
    }
}

impl<T> std::ops::Mul<f32> for Measurement<T>
where
    T: ScreenMeasurement,
{
    type Output = Self;

    fn mul(self, s: f32) -> Self {
        Self {
            value: T::from_f32(self.value.to_f32() * s),
        }
    }
}

impl<T> std::ops::Div<Self> for Measurement<T>
where
    T: ScreenMeasurement,
{
    type Output = Self;

    fn div(self, s: Self) -> Self {
        Self {
            value: T::from_f32(self.value.to_f32() / s.value.to_f32()),
        }
    }
}

impl<T> std::ops::Div<f32> for Measurement<T>
where
    T: ScreenMeasurement,
{
    type Output = Self;

    fn div(self, s: f32) -> Self {
        Self {
            value: T::from_f32(self.value.to_f32() / s),
        }
    }
}

impl<T> std::ops::Add<Self> for Measurement<T>
where
    T: ScreenMeasurement,
{
    type Output = Self;

    fn add(self, s: Self) -> Self {
        Self {
            value: T::from_f32(self.value.to_f32() + s.value.to_f32()),
        }
    }
}

impl<T> std::ops::Add<f32> for Measurement<T>
where
    T: ScreenMeasurement,
{
    type Output = Self;

    fn add(self, s: f32) -> Self {
        Self {
            value: T::from_f32(self.value.to_f32() + s),
        }
    }
}

impl<T> std::ops::Sub<Self> for Measurement<T>
where
    T: ScreenMeasurement,
{
    type Output = Self;

    fn sub(self, s: Self) -> Self {
        Self {
            value: T::from_f32(self.value.to_f32() - s.value.to_f32()),
        }
    }
}

impl<T> std::ops::Sub<f32> for Measurement<T>
where
    T: ScreenMeasurement,
{
    type Output = Self;

    fn sub(self, s: f32) -> Self {
        Self {
            value: T::from_f32(self.value.to_f32() - s),
        }
    }
}

impl<T> std::ops::Neg for Measurement<T>
where
    T: ScreenMeasurement,
{
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            value: T::from_f32(self.value.to_f32().neg()),
        }
    }
}

impl<T> std::ops::MulAssign<f32> for Measurement<T>
where
    T: ScreenMeasurement,
{
    fn mul_assign(&mut self, rhs: f32) {
        self.value = T::from_f32(self.value.to_f32() * rhs)
    }
}

impl<T> std::ops::MulAssign<Self> for Measurement<T>
where
    T: ScreenMeasurement,
{
    fn mul_assign(&mut self, rhs: Self) {
        self.value = T::from_f32(self.value.to_f32() * rhs.to_f32())
    }
}

impl<T> std::ops::DivAssign<f32> for Measurement<T>
where
    T: ScreenMeasurement,
{
    fn div_assign(&mut self, rhs: f32) {
        self.value = T::from_f32(self.value.to_f32() / rhs)
    }
}

impl<T> std::ops::DivAssign<Self> for Measurement<T>
where
    T: ScreenMeasurement,
{
    fn div_assign(&mut self, rhs: Self) {
        self.value = T::from_f32(self.value.to_f32() / rhs.to_f32())
    }
}

impl<T> std::ops::SubAssign<f32> for Measurement<T>
where
    T: ScreenMeasurement,
{
    fn sub_assign(&mut self, rhs: f32) {
        self.value = T::from_f32(self.value.to_f32() - rhs)
    }
}

impl<T> std::ops::SubAssign<Self> for Measurement<T>
where
    T: ScreenMeasurement,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.value = T::from_f32(self.value.to_f32() - rhs.to_f32())
    }
}

impl<T> std::ops::AddAssign<f32> for Measurement<T>
where
    T: ScreenMeasurement,
{
    fn add_assign(&mut self, rhs: f32) {
        self.value = T::from_f32(self.value.to_f32() + rhs)
    }
}

impl<T> std::ops::AddAssign<Self> for Measurement<T>
where
    T: ScreenMeasurement,
{
    fn add_assign(&mut self, rhs: Self) {
        self.value = T::from_f32(self.value.to_f32() + rhs.to_f32())
    }
}

impl<T> std::iter::Sum for Measurement<T>
where
    T: ScreenMeasurement + Default,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut out = Self::default();
        for value in iter {
            out += value;
        }
        out
    }
}
