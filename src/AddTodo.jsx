import { Controller, FormProvider, useForm, useFormContext } from "react-hook-form";
import { Button, FormControl, HStack, Input, Text, Textarea, VStack } from "@yamada-ui/react";
import {useEffect, useState} from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";

function AddTodo() {
    const form = useForm();
    const { register, handleSubmit, formState: {errors} } = form;
    const [ sendMessage, setSendMessage ] = useState("");
    const navi = useNavigate();

    const onSubmit = async (data) => { 
        const res = {item : {
            title : data.title,
            work : data.work,
            start : str2date(data.start)?.toLocaleDateString(),
            end : str2date(data.end)?.toLocaleDateString(),
        }};
        console.log(res);
        try {
            setSendMessage('送信中です。');
            await invoke('add_todo', res);
            navi('/todo');
        } catch (e) {
            setSendMessage('エラーが発生しました。{' + e + '}');
            console.log(e);
        }
    };

    return (
        <>
            <FormProvider {...form}>
                <VStack as="form" onSubmit={handleSubmit(onSubmit)}>
                    <FormControl
                        invalid={!!errors.title}
                        label="タイトル"
                        errorMessage={errors?.title?.message}
                    >
                        <Input placeholder="やること" 
                            {...register("title", {required:"入力は必須です。"})}/> 
                    </FormControl>
                    <FormControl label="詳細">
                        <Textarea {...register("work")} />
                    </FormControl>
                    <InputDate name="start" label="開始"/>
                    <InputDate name="end" label="終了"/>
                    <Button type="submit" w="30%" ml="auto" mr="auto">送信</Button>
                    <Text> {sendMessage} </Text>
                </VStack>
            </FormProvider>
        </>
    );
}

function InputDate({name, label}) {
    const { register, watch, formState: {errors} } = useFormContext();

    const val = watch(name);
    const [ date, setDate,  ] = useState(null);
    useEffect(() => {
        setDate(str2date(val));
    }, [val]);

    return (
        <>
            <FormControl 
                invalid = {!!errors[name]} 
                label={label} 
                errorMessage={errors[name]?.message}>
                <HStack>
                    <Input 
                        w="50%" 
                        placeholder="[[YYYY/]MM/]DD or +dd" 
                        {...register(name, {
                            validate: (data) => {
                            if (data==null) { return }
                            if (data.length===0) { return }
                            if (str2date(data)==null) { return "日付の形式が不正です。" }
                            }
                        })}
                    />
                    <Text> {date?.toLocaleDateString()} </Text>
                </HStack>
            </FormControl>
        </>
    );
}

function str2date(str) {
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

export default AddTodo;
