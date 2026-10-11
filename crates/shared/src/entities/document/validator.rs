use crate::errors::DocumentError;

pub fn sanitize_digits(val: &str) -> String {
    val.chars().filter(|c| c.is_ascii_digit()).collect()
}

pub fn validate_cnpj(val: &str) -> Result<(), DocumentError> {
    let digits = sanitize_digits(val);
    if digits.is_empty() {
        return Err(DocumentError::EmptyDocument);
    }
    if digits.len() != 14 {
        return Err(DocumentError::InvalidLength {
            expected: 14,
            received: digits.len(),
        });
    }

    // Verifica se todos os dígitos são iguais (ex: 00000000000000)
    let first_char = digits.chars().next().unwrap();
    if digits.chars().all(|c| c == first_char) {
        return Err(DocumentError::InvalidCheckDigits);
    }

    let nums: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();

    // 1º dígito verificador
    let weights1 = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum1: u32 = nums.iter().take(12).zip(weights1.iter()).map(|(&d, &w)| d * w).sum();
    let rest1 = sum1 % 11;
    let expected_d1 = if rest1 < 2 { 0 } else { 11 - rest1 };

    if nums[12] != expected_d1 {
        return Err(DocumentError::InvalidCheckDigits);
    }

    // 2º dígito verificador
    let weights2 = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum2: u32 = nums.iter().take(13).zip(weights2.iter()).map(|(&d, &w)| d * w).sum();
    let rest2 = sum2 % 11;
    let expected_d2 = if rest2 < 2 { 0 } else { 11 - rest2 };

    if nums[13] != expected_d2 {
        return Err(DocumentError::InvalidCheckDigits);
    }

    Ok(())
}

pub fn format_cnpj(val: &str) -> String {
    let digits = sanitize_digits(val);
    if digits.len() == 14 {
        format!(
            "{}.{}.{}/{}-{}",
            &digits[0..2],
            &digits[2..5],
            &digits[5..8],
            &digits[8..12],
            &digits[12..14]
        )
    } else {
        val.to_string()
    }
}

pub fn validate_cpf(val: &str) -> Result<(), DocumentError> {
    let digits = sanitize_digits(val);
    if digits.is_empty() {
        return Err(DocumentError::EmptyDocument);
    }
    if digits.len() != 11 {
        return Err(DocumentError::InvalidLength {
            expected: 11,
            received: digits.len(),
        });
    }

    let first_char = digits.chars().next().unwrap();
    if digits.chars().all(|c| c == first_char) {
        return Err(DocumentError::InvalidCheckDigits);
    }

    let nums: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();

    // 1º dígito verificador
    let sum1: u32 = nums.iter().take(9).enumerate().map(|(i, &d)| d * (10 - i as u32)).sum();
    let rest1 = sum1 % 11;
    let expected_d1 = if rest1 < 2 { 0 } else { 11 - rest1 };

    if nums[9] != expected_d1 {
        return Err(DocumentError::InvalidCheckDigits);
    }

    // 2º dígito verificador
    let sum2: u32 = nums.iter().take(10).enumerate().map(|(i, &d)| d * (11 - i as u32)).sum();
    let rest2 = sum2 % 11;
    let expected_d2 = if rest2 < 2 { 0 } else { 11 - rest2 };

    if nums[10] != expected_d2 {
        return Err(DocumentError::InvalidCheckDigits);
    }

    Ok(())
}

pub fn format_cpf(val: &str) -> String {
    let digits = sanitize_digits(val);
    if digits.len() == 11 {
        format!(
            "{}.{}.{}-{}",
            &digits[0..3],
            &digits[3..6],
            &digits[6..9],
            &digits[9..11]
        )
    } else {
        val.to_string()
    }
}

pub fn validate_cuit(val: &str) -> Result<(), DocumentError> {
    let digits = sanitize_digits(val);
    if digits.is_empty() {
        return Err(DocumentError::EmptyDocument);
    }
    if digits.len() != 11 {
        return Err(DocumentError::InvalidLength {
            expected: 11,
            received: digits.len(),
        });
    }

    let nums: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();

    let weights = [5, 4, 3, 2, 7, 6, 5, 4, 3, 2];
    let sum: u32 = nums.iter().take(10).zip(weights.iter()).map(|(&d, &w)| d * w).sum();
    let rest = sum % 11;

    let expected_digit = match rest {
        0 => 0,
        1 => 9, // Caso especial AFIP ou validação 11 - rest se aplicável
        r => 11 - r,
    };

    if nums[10] != expected_digit && (rest != 1 || nums[10] != 4) {
        return Err(DocumentError::InvalidCheckDigits);
    }

    Ok(())
}

pub fn format_cuit(val: &str) -> String {
    let digits = sanitize_digits(val);
    if digits.len() == 11 {
        format!("{}-{}-{}", &digits[0..2], &digits[2..10], &digits[10..11])
    } else {
        val.to_string()
    }
}

pub fn validate_dni(val: &str) -> Result<(), DocumentError> {
    let digits = sanitize_digits(val);
    if digits.is_empty() {
        return Err(DocumentError::EmptyDocument);
    }
    if digits.len() < 7 || digits.len() > 8 {
        return Err(DocumentError::InvalidLength {
            expected: 8,
            received: digits.len(),
        });
    }
    Ok(())
}

pub fn format_dni(val: &str) -> String {
    let digits = sanitize_digits(val);
    if digits.len() == 8 {
        format!("{}.{}.{}", &digits[0..2], &digits[2..5], &digits[5..8])
    } else if digits.len() == 7 {
        format!("{}.{}.{}", &digits[0..1], &digits[1..4], &digits[4..7])
    } else {
        val.to_string()
    }
}
