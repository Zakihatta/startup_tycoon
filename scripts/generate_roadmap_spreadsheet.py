"""
Generate Startup Tycoon Project Roadmap & Asset Registry Spreadsheet (.xlsx + .csv).
"""

import csv
import os
import openpyxl
from openpyxl.styles import Alignment, Border, Font, PatternFill, Side
from openpyxl.utils import get_column_letter
from openpyxl.worksheet.datavalidation import DataValidation


TASKS = [
    # M0 - Preproduction & Assets
    ("T-01", "Preproduction", "M0 - Preproduction", "Docs", "Susun dokumen visual direction, skala, & pipeline aset", "Must-Have", "Done", "docs/ASSETS_PREPARATION.md", "Skala 1m = 1 unit, style stylized modern 3D"),
    ("T-02", "Preproduction", "M0 - Preproduction", "Toolchain", "Install Rust, Cargo, & WinLibs MinGW-w64 MSVCRT", "Must-Have", "Done", "Cargo.toml; .cargo/config.toml", "rustc 1.100.0 GNU, jobs=2 untuk RAM 6GB"),
    ("T-03", "Preproduction", "M0 - Preproduction", "Asset 3D", "Bangun pipeline generator .glb prosedural", "High", "Done", "scripts/glb_lib.py", "Builder glTF 2.0 tervalidasi"),
    ("T-04", "Preproduction", "M0 - Preproduction", "Asset 3D", "Buat karakter 3D Founder (.glb) + animasi Idle & Walk", "High", "Done", "assets/characters/player/character_founder.glb", "Tinggi ~1.75m, hadap -Z, 2 animasi"),
    ("T-05", "Preproduction", "M0 - Preproduction", "Asset 3D", "Buat aset Environment modular M1 (floor, wall, window, door)", "Must-Have", "Done", "assets/environment/**/*.glb", "5 file .glb grid 1m, pintu pivot engsel"),
    ("T-06", "Preproduction", "M0 - Preproduction", "Asset 3D", "Buat aset Furniture & Equipment M2 (desk, chair, monitor, PC, KB+mouse)", "Must-Have", "Done", "assets/furniture/*.glb; assets/equipment/*.glb", "5 file .glb proporsional 1m"),

    # M1 - World & Player
    ("T-07", "MVP Core", "M1 - World & Player", "Engine/Code", "Inisialisasi Bevy 0.20 + auto-resolve path assets/", "Must-Have", "Done", "src/main.rs", "Jalan lewat cargo run & exe langsung"),
    ("T-08", "MVP Core", "M1 - World & Player", "Engine/Code", "Setup scene kantor awal 6m x 5m + lighting & shadows", "Must-Have", "Done", "src/office.rs", "Daylight directional + indoor point light"),
    ("T-09", "MVP Core", "M1 - World & Player", "Engine/Code", "Implementasi kontroler karakter (WASD, Shift, mouse look, collision)", "Must-Have", "Done", "src/player.rs", "Wall-sliding collision + bounds"),
    ("T-10", "MVP Core", "M1 - World & Player", "Engine/Code", "Tambah toggle kamera 1st/3rd Person (V) + animasi Idle/Walk", "High", "Done", "src/player.rs; src/main.rs", "AnimationGraph + AnimationTransitions"),

    # M2 - Office Interaction
    ("T-11", "MVP Core", "M2 - Office Interaction", "Engine/Code", "Tempatkan workstation awal di ruang kantor", "Must-Have", "Done", "src/office.rs", "Desk, chair, monitor, PC, keyboard+mouse"),
    ("T-12", "MVP Core", "M2 - Office Interaction", "Engine/Code", "Interaksi buka/tutup pintu dengan tombol E", "Must-Have", "Done", "src/office.rs", "Rotasi engsel mulus berdasarkan jarak"),
    ("T-13", "MVP Core", "M2 - Office Interaction", "UI", "Deteksi objek di depan crosshair + prompt UI dinamis ([E] ...)", "Must-Have", "Done", "src/interaction/mod.rs; src/ui/mod.rs", "Bekerja di mode 1st & 3rd Person"),
    ("T-14", "MVP Core", "M2 - Office Interaction", "UI", "Interaksi workstation/komputer untuk buka menu manajemen", "Must-Have", "Done", "src/ui/mod.rs; src/player.rs", "Modal StartupOS v0.1 + lepas cursor otomatis"),

    # M3 - Company & Finance
    ("T-15", "MVP Core", "M3 - Company & Finance", "Engine/Code", "Sistem waktu simulasi (jam/hari/bulan, Pause, 1x, 2x)", "Must-Have", "Done", "src/core/time.rs", "Tick 15m independen framerate + 3 unit test"),
    ("T-16", "MVP Core", "M3 - Company & Finance", "Engine/Code", "State perusahaan & keuangan (kas, sewa, gaji, pemasukan)", "Must-Have", "Done", "src/company/mod.rs; src/economy/mod.rs", "Ledger transaksi + sewa harian + 2 unit test"),
    ("T-17", "MVP Core", "M3 - Company & Finance", "Asset 2D/UI", "Siapkan aset UI icons (.png) & UI font (.ttf/.otf)", "Must-Have", "In Progress", "assets/ui/icons/; scripts/generate_ui_icons.py", "6 ikon PNG selesai; font masih pakai default Bevy"),
    ("T-18", "MVP Core", "M3 - Company & Finance", "UI", "HUD utama (uang, tanggal, speed) + panel Finance/Report", "Must-Have", "Done", "src/ui/mod.rs", "HUD top-right + tab Overview & Finance di StartupOS"),

    # M4 - Product Development
    ("T-19", "MVP Core", "M4 - Product Development", "Engine/Code", "Model data & progres produk (fitur, kualitas, bug, rilis versi)", "Must-Have", "To Do", "src/product.rs (planned)", "Siklus build & release produk software"),
    ("T-20", "MVP Core", "M4 - Product Development", "UI", "UI Product Development di layar komputer/manajemen", "Must-Have", "To Do", "src/ui.rs (planned)", "Pilih proyek, pantau progres, rilis"),

    # M5 - Employee System
    ("T-21", "MVP Core", "M5 - Employee System", "Asset 3D", "Buat karakter 3D Developer (.glb) + animasi Idle, Walk, Sitting, Typing", "Must-Have", "To Do", "assets/characters/employees/character_developer.glb", "Skeleton/rig konsisten dengan Founder"),
    ("T-22", "MVP Core", "M5 - Employee System", "Asset 2D/UI", "Buat Employee Portrait (.png) untuk daftar kandidat & profil", "Must-Have", "To Do", "assets/characters/employees/portraits/", "Format PNG transparan/crop render"),
    ("T-23", "MVP Core", "M5 - Employee System", "Engine/Code", "Sistem rekrutmen (hiring), gaji, & assign ke workstation", "Must-Have", "To Do", "src/employee.rs (planned)", "Karyawan menambah kecepatan dev produk"),
    ("T-24", "MVP Core", "M5 - Employee System", "Engine/Code", "AI NPC sederhana (jalan ke kursi, duduk, animasi mengetik)", "High", "To Do", "src/employee.rs (planned)", "Navigasi sederhana dari pintu ke desk"),

    # M6 - Market & Revenue
    ("T-25", "MVP Core", "M6 - Market & Revenue", "Engine/Code", "Simulasi pasar, pertumbuhan user, churn, & pendapatan produk", "Must-Have", "To Do", "src/market.rs (planned)", "Menghubungkan kualitas produk ke revenue"),
    ("T-26", "MVP Core", "M6 - Market & Revenue", "UI", "UI Market & ringkasan tren pendapatan", "High", "To Do", "src/ui.rs (planned)", "Grafik/statistik sederhana"),

    # M7 - Management UI & MVP Integration
    ("T-27", "MVP Core", "M7 - MVP Integration", "UI", "Integrasi penuh panel Dashboard, Finance, Product, Employee, Hiring, Market", "Must-Have", "To Do", "src/ui.rs (planned)", "Navigasi tab mulus di UI komputer"),
    ("T-28", "MVP Core", "M7 - MVP Integration", "QA/Release", "Balancing ekonomi & playtest penuh gameplay loop MVP", "Must-Have", "To Do", "-", "Plan -> Build -> Operate -> Analyze"),

    # M8 - Office Expansion
    ("T-29", "Post-MVP", "M8 - Office Expansion", "Asset 3D", "Tambah aset furniture lanjutan (sofa, meeting table, whiteboard, tanaman, server)", "Medium", "To Do", "assets/furniture/; assets/equipment/", "Sesuai daftar Future di ASSETS_PREPARATION.md"),
    ("T-30", "Post-MVP", "M8 - Office Expansion", "Engine/Code", "Sistem beli/sewa ruang baru & grid building placement (1m)", "Medium", "To Do", "src/building.rs (planned)", "Pasang desk/chair/PC baru secara dinamis"),

    # M9 - Save System (Included in MVP per docs/GDD/development/mvp_scope.md)
    ("T-31", "MVP Core", "M9 - Save System", "Engine/Code", "Implementasi Save/Load state perusahaan, karyawan, produk, & layout", "Must-Have", "To Do", "src/save_system/ (planned)", "Serialisasi serde JSON/RON + validasi aset"),

    # M10 - Polish & Audio
    ("T-32", "Post-MVP", "M10 - Polish & Audio", "Audio", "Siapkan & integrasikan audio .ogg (UI click, footsteps, typing, door, ambience)", "Low", "To Do", "assets/audio/", "Bevy Audio / rodio"),
    ("T-33", "Post-MVP", "M10 - Polish & Audio", "QA/Release", "Optimasi performa, perapihan ceiling/detail visual, & bugfixing", "Medium", "To Do", "-", "Evaluasi FPS & ukuran aset"),

    # Deployment & Launch
    ("T-34", "Deployment & Launch", "Release & Launch", "QA/Release", "Compile release build (cargo build --release) untuk Windows/Linux/macOS", "Must-Have", "To Do", "target/release/startup_tycon.exe", "Opsional: WASM web demo"),
    ("T-35", "Deployment & Launch", "Release & Launch", "QA/Release", "Packaging folder rilis (executable + assets/ + lisensi) & uji di PC bersih", "Must-Have", "To Do", "-", "Pastikan tanpa dependency Rust eksternal"),
    ("T-36", "Deployment & Launch", "Release & Launch", "QA/Release", "Siapkan halaman storefront (itch.io / Steam): screenshot, trailer, deskripsi", "Must-Have", "To Do", "-", "Rilis v0.1 MVP/Demo & kumpulkan feedback"),
]

ASSETS = [
    ("Founder Character", "Character", "Procedural (scripts/generate_character_glb.py)", "GLB", "Project Original", "Tested", "assets/characters/player/character_founder.glb", "Idle + Walk animations, 1.75m"),
    ("Office Floor Tile", "Environment", "Procedural (scripts/generate_office_assets.py)", "GLB", "Project Original", "Tested", "assets/environment/floor/environment_floor_tile.glb", "1m x 1m modular tile"),
    ("Wall Plain", "Environment", "Procedural (scripts/generate_office_assets.py)", "GLB", "Project Original", "Tested", "assets/environment/walls/environment_wall_plain.glb", "1m x 2.8m x 0.15m"),
    ("Wall Window", "Environment", "Procedural (scripts/generate_office_assets.py)", "GLB", "Project Original", "Tested", "assets/environment/walls/environment_wall_window.glb", "Kaca BLEND transparan"),
    ("Wall Door Frame", "Environment", "Procedural (scripts/generate_office_assets.py)", "GLB", "Project Original", "Tested", "assets/environment/walls/environment_wall_door.glb", "Kusen 0.84m x 2.12m"),
    ("Door Basic", "Environment", "Procedural (scripts/generate_office_assets.py)", "GLB", "Project Original", "Tested", "assets/environment/doors/environment_door_basic.glb", "Engsel pivot di x=0"),
    ("Desk Basic", "Furniture", "Procedural (scripts/generate_office_assets.py)", "GLB", "Project Original", "Tested", "assets/furniture/furniture_desk_basic.glb", "1.4m x 0.7m x 0.75m"),
    ("Office Chair", "Furniture", "Procedural (scripts/generate_office_assets.py)", "GLB", "Project Original", "Tested", "assets/furniture/furniture_chair_office.glb", "Seat y=0.45m, roda 5-star"),
    ("Monitor Basic", "Equipment", "Procedural (scripts/generate_office_assets.py)", "GLB", "Project Original", "Tested", "assets/equipment/equipment_monitor_basic.glb", "0.62m lebar, emissive screen"),
    ("PC Basic", "Equipment", "Procedural (scripts/generate_office_assets.py)", "GLB", "Project Original", "Tested", "assets/equipment/equipment_pc_basic.glb", "Mid-tower dengan LED aksen"),
    ("Keyboard & Mouse", "Equipment", "Procedural (scripts/generate_office_assets.py)", "GLB", "Project Original", "Tested", "assets/equipment/equipment_keyboard_mouse.glb", "Set periferal meja"),
    ("Developer Character", "Character", "TBD", "GLB", "TBD", "Needed", "assets/characters/employees/character_developer.glb", "Idle, Walk, Sitting, Typing (M5)"),
    ("Employee Portraits", "UI", "TBD", "PNG", "TBD", "Needed", "assets/characters/employees/portraits/", "Untuk UI kandidat & profil (M5)"),
    ("UI Icon Set", "UI", "Procedural (scripts/generate_ui_icons.py)", "PNG", "Project Original", "Tested", "assets/ui/icons/", "6 ikon 64x64 RGBA: money, time, company, employee, product, settings"),
    ("UI Font", "UI", "TBD", "TTF/OTF", "TBD", "Needed", "assets/fonts/", "Font UI berlisensi komersial (M3)"),
    ("Furniture Lanjutan", "Furniture", "TBD", "GLB", "TBD", "Needed", "assets/furniture/", "Sofa, meeting table, whiteboard, tanaman (M8)"),
    ("Audio SFX & Ambience", "Audio", "TBD", "OGG", "TBD", "Needed", "assets/audio/", "UI click, footsteps, typing, door, BGM (M10)"),
]

MILESTONES = [
    "M0 - Preproduction",
    "M1 - World & Player",
    "M2 - Office Interaction",
    "M3 - Company & Finance",
    "M4 - Product Development",
    "M5 - Employee System",
    "M6 - Market & Revenue",
    "M7 - MVP Integration",
    "M8 - Office Expansion",
    "M9 - Save System",
    "M10 - Polish & Audio",
    "Release & Launch",
]


def style_header(cell, fill_hex="1F3A5F"):
    cell.font = Font(name="Calibri", size=11, bold=True, color="FFFFFF")
    cell.fill = PatternFill("solid", fgColor=fill_hex)
    cell.alignment = Alignment(horizontal="center", vertical="center", wrap_text=True)


def thin_border():
    side = Side(border_style="thin", color="D0D7DE")
    return Border(left=side, right=side, top=side, bottom=side)


def status_style(status: str):
    if status == "Done" or status == "Tested":
        return PatternFill("solid", fgColor="DCFCE7"), Font(name="Calibri", size=10, bold=True, color="166534")
    if status == "In Progress":
        return PatternFill("solid", fgColor="FEF3C7"), Font(name="Calibri", size=10, bold=True, color="92400E")
    return PatternFill("solid", fgColor="F1F5F9"), Font(name="Calibri", size=10, bold=False, color="334155")


def build_spreadsheet(xlsx_path: str, csv_path: str) -> None:
    wb = openpyxl.Workbook()

    # ------------------------------------------------------------------
    # Sheet 1: Dashboard (Summary with live Excel COUNTIF/COUNTIFS formulas)
    # ------------------------------------------------------------------
    ws_dash = wb.active
    ws_dash.title = "Dashboard"
    ws_dash.views.sheetView[0].showGridLines = True

    ws_dash["A1"] = "Startup Tycoon — Project Tracker & Roadmap (Rust + Bevy)"
    ws_dash["A1"].font = Font(name="Calibri", size=16, bold=True, color="1F3A5F")
    ws_dash["A2"] = "Update kolom Status di sheet 'To-Do List' (Done / In Progress / To Do); angka progres di bawah akan otomatis terhitung."
    ws_dash["A2"].font = Font(name="Calibri", size=10, italic=True, color="475569")

    # KPI Cards row 4-5
    last_task_row = 1 + len(TASKS)
    kpis = [
        ("A", "Total Task", f"=COUNTA('To-Do List'!A2:A{last_task_row})", "0"),
        ("B", "Selesai (Done)", f"=COUNTIF('To-Do List'!G2:G{last_task_row}, \"Done\")", "0"),
        ("C", "Sedang Dikerjakan", f"=COUNTIF('To-Do List'!G2:G{last_task_row}, \"In Progress\")", "0"),
        ("D", "Belum (To Do)", f"=COUNTIF('To-Do List'!G2:G{last_task_row}, \"To Do\")", "0"),
        ("E", "Progress Total", "=IF(A5=0,0,B5/A5)", "0.0%"),
    ]
    for col, label, formula, num_fmt in kpis:
        lbl_cell = ws_dash[f"{col}4"]
        lbl_cell.value = label
        style_header(lbl_cell, "2B4C7E")
        lbl_cell.border = thin_border()

        val_cell = ws_dash[f"{col}5"]
        val_cell.value = formula
        val_cell.font = Font(name="Calibri", size=14, bold=True, color="0F172A")
        val_cell.alignment = Alignment(horizontal="center", vertical="center")
        val_cell.number_format = num_fmt
        val_cell.fill = PatternFill("solid", fgColor="F8FAFC")
        val_cell.border = thin_border()

    ws_dash.row_dimensions[4].height = 24
    ws_dash.row_dimensions[5].height = 30

    # Milestone breakdown table starting at row 7
    headers_dash = ["Milestone", "Total Task", "Done", "In Progress", "To Do", "% Selesai"]
    for idx, h in enumerate(headers_dash, start=1):
        c = ws_dash.cell(row=7, column=idx, value=h)
        style_header(c)
        c.border = thin_border()
    ws_dash.row_dimensions[7].height = 24

    for m_idx, ms in enumerate(MILESTONES, start=8):
        ws_dash.cell(row=m_idx, column=1, value=ms).font = Font(name="Calibri", size=10, bold=True)
        ws_dash.cell(row=m_idx, column=2, value=f"=COUNTIF('To-Do List'!$C$2:$C${last_task_row}, A{m_idx})")
        ws_dash.cell(row=m_idx, column=3, value=f"=COUNTIFS('To-Do List'!$C$2:$C${last_task_row}, A{m_idx}, 'To-Do List'!$G$2:$G${last_task_row}, \"Done\")")
        ws_dash.cell(row=m_idx, column=4, value=f"=COUNTIFS('To-Do List'!$C$2:$C${last_task_row}, A{m_idx}, 'To-Do List'!$G$2:$G${last_task_row}, \"In Progress\")")
        ws_dash.cell(row=m_idx, column=5, value=f"=COUNTIFS('To-Do List'!$C$2:$C${last_task_row}, A{m_idx}, 'To-Do List'!$G$2:$G${last_task_row}, \"To Do\")")
        pct_cell = ws_dash.cell(row=m_idx, column=6, value=f"=IF(B{m_idx}=0, 0, C{m_idx}/B{m_idx})")
        pct_cell.number_format = "0%"

        for col_i in range(1, 7):
            cell = ws_dash.cell(row=m_idx, column=col_i)
            cell.border = thin_border()
            if col_i > 1:
                cell.alignment = Alignment(horizontal="center", vertical="center")
                cell.font = Font(name="Calibri", size=10)
            else:
                cell.alignment = Alignment(horizontal="left", vertical="center")
        ws_dash.row_dimensions[m_idx].height = 21

    dash_widths = [28, 16, 16, 18, 16, 16]
    for i, w in enumerate(dash_widths, start=1):
        ws_dash.column_dimensions[get_column_letter(i)].width = w

    # ------------------------------------------------------------------
    # Sheet 2: To-Do List
    # ------------------------------------------------------------------
    ws_todo = wb.create_sheet("To-Do List")
    todo_headers = [
        "ID",
        "Fase",
        "Milestone",
        "Kategori",
        "Task / Pekerjaan",
        "Prioritas",
        "Status",
        "Output / File Terkait",
        "Catatan",
    ]
    for idx, h in enumerate(todo_headers, start=1):
        c = ws_todo.cell(row=1, column=idx, value=h)
        style_header(c)
        c.border = thin_border()
    ws_todo.row_dimensions[1].height = 26

    for r_idx, row in enumerate(TASKS, start=2):
        for c_idx, val in enumerate(row, start=1):
            c = ws_todo.cell(row=r_idx, column=c_idx, value=val)
            c.border = thin_border()
            c.font = Font(name="Calibri", size=10)
            if c_idx in (1, 4, 6, 7):
                c.alignment = Alignment(horizontal="center", vertical="center", wrap_text=True)
            else:
                c.alignment = Alignment(horizontal="left", vertical="center", wrap_text=True)

            if c_idx == 7:
                fill, font = status_style(val)
                c.fill = fill
                c.font = font
        ws_todo.row_dimensions[r_idx].height = 24

    ws_todo.freeze_panes = "A2"
    ws_todo.auto_filter.ref = f"A1:I{last_task_row}"

    dv_status = DataValidation(type="list", formula1='"Done,In Progress,To Do"', allow_blank=False)
    ws_todo.add_data_validation(dv_status)
    dv_status.add(f"G2:G{last_task_row}")

    todo_widths = [8, 18, 22, 13, 52, 12, 13, 38, 36]
    for i, w in enumerate(todo_widths, start=1):
        ws_todo.column_dimensions[get_column_letter(i)].width = w

    # ------------------------------------------------------------------
    # Sheet 3: Asset Registry
    # ------------------------------------------------------------------
    ws_assets = wb.create_sheet("Asset Registry")
    asset_headers = ["Asset Name", "Category", "Source", "Format", "License", "Status", "Path / Target", "Notes"]
    for idx, h in enumerate(asset_headers, start=1):
        c = ws_assets.cell(row=1, column=idx, value=h)
        style_header(c)
        c.border = thin_border()
    ws_assets.row_dimensions[1].height = 26

    for r_idx, row in enumerate(ASSETS, start=2):
        for c_idx, val in enumerate(row, start=1):
            c = ws_assets.cell(row=r_idx, column=c_idx, value=val)
            c.border = thin_border()
            c.font = Font(name="Calibri", size=10)
            if c_idx in (2, 4, 6):
                c.alignment = Alignment(horizontal="center", vertical="center", wrap_text=True)
            else:
                c.alignment = Alignment(horizontal="left", vertical="center", wrap_text=True)
            if c_idx == 6:
                fill, font = status_style(val)
                c.fill = fill
                c.font = font
        ws_assets.row_dimensions[r_idx].height = 22

    last_asset_row = 1 + len(ASSETS)
    ws_assets.freeze_panes = "A2"
    ws_assets.auto_filter.ref = f"A1:H{last_asset_row}"

    dv_asset = DataValidation(type="list", formula1='"Needed,Searching,Selected,Imported,Tested,Rejected"', allow_blank=False)
    ws_assets.add_data_validation(dv_asset)
    dv_asset.add(f"F2:F{last_asset_row}")

    asset_widths = [22, 14, 38, 10, 18, 12, 46, 36]
    for i, w in enumerate(asset_widths, start=1):
        ws_assets.column_dimensions[get_column_letter(i)].width = w

    os.makedirs(os.path.dirname(os.path.abspath(xlsx_path)), exist_ok=True)
    wb.save(xlsx_path)

    # Also export UTF-8-SIG CSV for quick Google Sheets / plain-text import
    with open(csv_path, "w", newline="", encoding="utf-8-sig") as f:
        writer = csv.writer(f)
        writer.writerow(todo_headers)
        writer.writerows(TASKS)


if __name__ == "__main__":
    xlsx_out = os.path.join("docs", "Startup_Tycoon_Roadmap.xlsx")
    csv_out = os.path.join("docs", "Startup_Tycoon_Roadmap.csv")
    build_spreadsheet(xlsx_out, csv_out)
    print(f"Generated {xlsx_out} ({os.path.getsize(xlsx_out)} bytes)")
    print(f"Generated {csv_out} ({os.path.getsize(csv_out)} bytes)")
