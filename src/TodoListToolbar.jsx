import { useNavigate } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { HStack, IconButton, Select, Switch, Option } from "@yamada-ui/react";
import { invoke } from "@tauri-apps/api/core";
import { AiOutlineFileAdd } from "react-icons/ai";
import { useState } from "react";
import "./App.css";


export default function TodoListToolbar() {

    const navi = useNavigate();
    const handleAddTodo = () => navi('/addtodo');

    const queryClient = useQueryClient();
    const [ IsIncomplete, setIsIncomplete ] = useState(true);
    const onIsIncompleteChange = async (e) => {
        setIsIncomplete(e.target.checked);
        await invoke("set_is_incomplete", {isIncomplete: e.target.checked});
        queryClient.invalidateQueries({ queryKey: ['todo_list']});
    }; 

    return (
        <>
            <HStack>
                <IconButton icon={<AiOutlineFileAdd/>} onClick={handleAddTodo}/>
                <Switch checked={IsIncomplete} onChange={onIsIncompleteChange}>
                    未完了のみ
                </Switch>
                <SelectItemSortOrder/>
            </HStack>
        </>
    );
}

const getItemSortOrder = async () => invoke('get_item_sort_order') ;
const setItemSortOrder = 
    async (sortOrder) => invoke('set_item_sort_order', {sortOrder: sortOrder}) ;

function SelectItemSortOrder() {
    const {data, isPending} = useQuery({
        queryKey: ['item_sort_order'],
        queryFn: getItemSortOrder,
    });

    const queryClient = useQueryClient();

    const {mutate} = useMutation({
        mutationFn: setItemSortOrder,
        onSuccess: () => {
            queryClient.invalidateQueries({queryKey: ['item_sort_order']});
            queryClient.invalidateQueries({queryKey: ['todo_list']});
        },
        onError: (err) => console.log(err),
    });

    const onChange = (value) =>  mutate(value) ;

    if (isPending) {
        return (<p> loading </p>);
    }

    return (
        <Select w="9em" value={data} onChange={onChange}>
            <Option value="StartAsc">開始(昇順)</Option>
            <Option value="StartDesc">開始(降順)</Option>
            <Option value="EndAsc">終了(昇順)</Option>
            <Option value="EndDesc">終了(降順)</Option>
            <Option value="UpdateAsc">更新日(昇順)</Option>
            <Option value="UpdateDesc">更新日(降順)</Option>
        </Select>
    );
}
