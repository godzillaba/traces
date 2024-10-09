use anyhow::{Context, Result};
use rust_xlsxwriter::*;

const TARGETS_FILE: &str = "./targets.csv";
const DATA_FILE: &str = "./data/pretty-traces.tsv";

#[derive(Debug, PartialEq, Eq)]
struct Target {
    address: String,
    label: String,
}

// this is in the tsv
#[derive(Debug)]
struct PrettyCall {
    call_type: String,
    signature: String,
    from: String,
    to: String,
    from_name: String,
    to_name: String,
}

fn load_targets() -> Result<Vec<Target>> {
    let targets = std::fs::read_to_string(TARGETS_FILE)
        .context("Failed to read targets file")?
        .lines()
        .map(|line| {
            let arr = line.split(',').collect::<Vec<&str>>();

            Target {
                address: arr[0].to_string().to_lowercase(),
                label: arr[1].to_string(),
            }
        })
        .collect::<Vec<Target>>();

    Ok(targets)
}

fn load_data() -> Result<Vec<PrettyCall>> {
    let data = std::fs::read_to_string(DATA_FILE)
        .context("Failed to read data file")?
        .lines()
        .map(|line| {
            let arr = line.split('\t').collect::<Vec<&str>>();

            PrettyCall {
                call_type: arr[0].to_string(),
                signature: arr[1].to_string(),
                from: arr[2].to_string(),
                to: arr[3].to_string(),
                from_name: arr[4].to_string(),
                to_name: arr[5].to_string(),
            }
        })
        .filter(|call| call.call_type != "toplevel")
        .collect::<Vec<PrettyCall>>();

    Ok(data)
}

fn create_sheet_for_target(
    target: &Target,
    targets: &Vec<Target>,
    data: &Vec<PrettyCall>,
    workbook: &mut Workbook,
) -> Result<()> {
    let sheet = workbook.add_worksheet();

    let default_format = Format::new().set_font_name("Roboto Mono").clone();
    let bold_format = Format::new()
        .set_font_name("Roboto Mono")
        .set_bold()
        .clone();
    let target_address_format = Format::new()
        .set_font_name("Roboto Mono")
        .set_background_color(Color::RGB(0xb6d7a8))
        .clone();
    let other_target_address_format = Format::new()
        .set_font_name("Roboto Mono")
        .set_background_color(Color::RGB(0x9fc5e8))
        .clone();

    // Write Title Info
    sheet.set_name(&target.label)?;
    sheet.write_string_with_format(0, 0, "Target Address", &bold_format)?;
    sheet.write_string_with_format(0, 1, "Target Label", &bold_format)?;
    sheet.write_string_with_format(1, 0, &target.address, &default_format)?;
    sheet.write_string_with_format(1, 1, &target.label, &default_format)?;

    sheet.write_string_with_format(3, 0, "Call Type", &bold_format)?;
    sheet.write_string_with_format(3, 1, "Signature", &bold_format)?;
    sheet.write_string_with_format(3, 2, "From", &bold_format)?;
    sheet.write_string_with_format(3, 3, "To", &bold_format)?;
    sheet.write_string_with_format(3, 4, "From Name", &bold_format)?;
    sheet.write_string_with_format(3, 5, "To Name", &bold_format)?;

    let data_start_row = 4;

    // Filter and sort data
    let mut to_matches = data
        .iter()
        .filter(|call| call.to == target.address)
        .collect::<Vec<&PrettyCall>>();
    let mut from_matches = data
        .iter()
        .filter(|call| call.from == target.address)
        .collect::<Vec<&PrettyCall>>();
    to_matches.sort_by(|a, b| b.from_name.cmp(&a.from_name));
    from_matches.sort_by(|a, b| b.to_name.cmp(&a.to_name));
    to_matches.append(&mut from_matches);
    let all_records = to_matches;

    for (row, call) in all_records.iter().enumerate() {
        let row_u32: u32 = row.try_into().unwrap();
        sheet.write_string_with_format(
            data_start_row + row_u32,
            0,
            &call.call_type,
            &default_format,
        )?;
        sheet.write_string_with_format(
            data_start_row + row_u32,
            1,
            &call.signature,
            &default_format,
        )?;
        sheet.write_string_with_format(data_start_row + row_u32, 2, &call.from, &default_format)?;
        sheet.write_string_with_format(data_start_row + row_u32, 3, &call.to, &default_format)?;
        sheet.write_string_with_format(
            data_start_row + row_u32,
            4,
            &call.from_name,
            &default_format,
        )?;
        sheet.write_string_with_format(
            data_start_row + row_u32,
            5,
            &call.to_name,
            &default_format,
        )?;

        // check if to is a different target
        if targets.iter().any(|t| t.address == call.to) {
            sheet.set_cell_format(data_start_row + row_u32, 3, &other_target_address_format)?;
            sheet.set_cell_format(data_start_row + row_u32, 5, &other_target_address_format)?;
        }

        // check if from is a different target
        if targets.iter().any(|t| t.address == call.from) {
            sheet.set_cell_format(data_start_row + row_u32, 2, &other_target_address_format)?;
            sheet.set_cell_format(data_start_row + row_u32, 4, &other_target_address_format)?;
        }

        if call.to == target.address {
            sheet.set_cell_format(data_start_row + row_u32, 3, &target_address_format)?;
            sheet.set_cell_format(data_start_row + row_u32, 5, &target_address_format)?;
        } else {
            sheet.set_cell_format(data_start_row + row_u32, 2, &target_address_format)?;
            sheet.set_cell_format(data_start_row + row_u32, 4, &target_address_format)?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    // Create a new workbook
    let mut workbook = Workbook::new();

    // load the data
    let data = load_data()?;

    // load the targets
    let targets = load_targets()?;

    // create the sheets
    for target in targets.iter() {
        create_sheet_for_target(target, &targets, &data, &mut workbook)?;
    }

    workbook.save("./data/spreadsheet.xlsx")?;

    Ok(())
}
