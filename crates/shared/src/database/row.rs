use crate::entities::BusinessError;
use sqlx::{ColumnIndex, Decode, Row, Type};

/// Extrai um campo tipado de uma linha do banco de dados (`Row`), mapeando eventuais erros para `BusinessError`.
pub fn row_get_field<'r, T, R, I>(row: &'r R, index: I) -> Result<T, BusinessError>
where
    R: Row,
    I: ColumnIndex<R> + std::fmt::Display + Copy,
    T: Decode<'r, R::Database> + Type<R::Database>,
{
    row.try_get(index).map_err(|err| {
        BusinessError::internal(format!("database.column.error[{index}|{err}]"))
    })
}

/// Trait de extensão para conveniência no encadeamento de chamadas em linhas de banco de dados (`Row`).
pub trait RowExt {
    fn get_field<'r, T, I>(&'r self, index: I) -> Result<T, BusinessError>
    where
        Self: Row,
        I: ColumnIndex<Self> + std::fmt::Display + Copy,
        T: Decode<'r, <Self as Row>::Database> + Type<<Self as Row>::Database>;
}

impl<R: Row> RowExt for R {
    fn get_field<'r, T, I>(&'r self, index: I) -> Result<T, BusinessError>
    where
        Self: Row,
        I: ColumnIndex<Self> + std::fmt::Display + Copy,
        T: Decode<'r, <Self as Row>::Database> + Type<<Self as Row>::Database>,
    {
        row_get_field(self, index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgRow;

    #[test]
    fn test_row_get_field_signature_and_trait() {
        // Verifica que o trait RowExt e a função row_get_field funcionam para PgRow
        fn _assert_row_ext(row: &PgRow) {
            let _ = row.get_field::<i64, _>("id");
            let _ = row_get_field::<i64, _, _>(row, "id");
        }
    }
}
