import csv

# Convert AT-HYG v2.4 database to encodable csv format.

file = csv.reader(open("athyg_v24.csv"))

writer = csv.writer(open("athyg_v24_processed.csv", 'w'))

tyc_idx = 0
gaia_idx = 0
hyg_idx = 0
hip_idx = 0
hd_idx = 0
hr_idx = 0
proper_idx = 0
x0_idx = 0
y0_idx = 0
z0_idx = 0
colour_index_idx = 0
mag_src_idx = 0
abs_mag_idx = 0

header = None
for line in file:
    if header is None:
        header = line
        tyc_idx = header.index('tyc')
        gaia_idx = header.index('gaia')
        hyg_idx = header.index('hyg')
        hip_idx = header.index('hip')
        hd_idx = header.index('hd')
        hr_idx = header.index('hr')
        proper_idx = header.index('proper')
        x0_idx = header.index('x0')
        y0_idx = header.index('y0')
        z0_idx = header.index('z0')
        colour_index_idx = header.index('ci')
        mag_src_idx = header.index('mag_src')
        abs_mag_idx = header.index('absmag')
        
        writer.writerow(['name', 'x', 'y', 'z', 'colour_index', 'abs_mag'])
        continue
    
    name = ""
    if len(line[proper_idx]) > 0:
        name = line[proper_idx]
    elif len(line[hd_idx]) > 0:
        name = "HD "+line[hd_idx]
    elif len(line[hr_idx]) > 0:
        name = "HR "+line[hr_idx]
    elif len(line[hip_idx]) > 0:
        name = "HIP "+line[hip_idx]
    elif len(line[hyg_idx]) > 0:
        name = "HYG "+line[hyg_idx]
    elif len(line[gaia_idx]) > 0:
        name = "GAIA "+line[gaia_idx]
    elif len(line[tyc_idx]) > 0:
        name = "TYC "+line[tyc_idx]
    else:
        print(line)
        raise Exception("no name")
    if len(line[x0_idx]) == 0 or len(line[colour_index_idx]) == 0:
        continue
    x = float(line[x0_idx])
    y = float(line[y0_idx])
    z = float(line[z0_idx])

    mag_src = line[mag_src_idx]
    colour_index = float(line[colour_index_idx]) * (0.85 if mag_src == "T" else 1.0) # convert BT-VT to B-V by multiplying by 0.85 (https://www.cosmos.esa.int/documents/532822/552851/vol1_all.pdf, section 1.3, equation 1.3.20, pg. 57)

    abs_mag = float(line[abs_mag_idx])
    writer.writerow([name, x, y, z, colour_index, abs_mag])

