import { invoke } from "@tauri-apps/api/core";
import { str2date } from "./str2date.jsx";
import { InputTodo } from "./InputTodo.jsx";

function AddTodo() {

    const send_data = async (data) => {
        const res = {item : {
            title : data.title,
            work : data.work,
            start : str2date(data.start)?.toLocaleDateString(),
            end : str2date(data.end)?.toLocaleDateString(),
        }};
        await invoke('add_todo', res);
    };

    const init_val = {
        title : "",
        work : "",
        start : "",
        end : "",
    };

    return (
        <>
            <InputTodo send_data={send_data} init_val={init_val}/>
        </>
    );
}

export default AddTodo;
