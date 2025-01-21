// 日付処理ユーティリティ

export function str2date(str) {
    if (str == null) { return null; }
    if (str.length === 0) { return null; }
    const date_item = str.split('/');
    for (const s of date_item) {
        if (Number.isNaN(Number(s))) { return null; }
    }

    const cur_date = new Date();
    const cur_year = cur_date.getFullYear();

    let ret_date = cur_date;
    try {
        switch (date_item.length) {
            case 0:
                return null;
            case 1:
                if (date_item[0][0] == '+') {
                    ret_date.setDate(ret_date.getDate() + Number(date_item[0]));
                } else {
                    ret_date.setDate(Number(date_item[0]));
                    if (ret_date < new Date()) {
                        ret_date.setMonth(ret_date.getMonth() + 1);
                    }
                }

                break;
            case 2:
                ret_date = new Date(cur_year, Number(date_item[0])-1, Number(date_item[1]))
                if (ret_date < new Date()) {
                    ret_date.setFullYear(ret_date.getFullYear() + 1);
                }
                break;
            case 3:
                const year = Number(date_item[0]);
                const month = Number(date_item[1]);
                const date = Number(date_item[2]);
                ret_date = new Date(year, month-1, date);
                break;
            default:
                return null;
        }
    } catch(e) {
        return null;
    }
    if (Number.isNaN(ret_date.getTime())) { return null; }

    return ret_date;
}
