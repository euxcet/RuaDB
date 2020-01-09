tables = ["customer", "lineitem", "nation", "orders", "part", "partsupp", "region", "supplier"]

for table in tables:
    csv = "../dataset/dataset_small/" + table + ".csv"
    fin = open(csv, "r")
    lines = fin.readlines()
    for line in lines:
        values = line.split(',')
        print("INSERT INTO" + table.upper() + " VALUES (", )
        print(")")
    print(csv)
