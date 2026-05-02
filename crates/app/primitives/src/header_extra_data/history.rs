pub trait HeaderExtraDataHistory {
    type Error;

    fn header_extra_data_at_height(&self, height: u64) -> Result<Option<Vec<u8>>, Self::Error>;
}
