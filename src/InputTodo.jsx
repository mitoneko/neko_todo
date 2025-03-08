import { FormProvider, useForm, useFormContext } from "react-hook-form";
import { Button, Container, FormControl, HStack, Input, Text, Textarea, VStack } from "@yamada-ui/react";
import {useEffect, useState} from "react";
import { useNavigate } from "react-router-dom";
import { useMutation } from "@tanstack/react-query";
import { str2date } from "./str2date.jsx";

export function InputTodo({send_data, init_val}) {
    const form = useForm({
        defaultValues: {
            title: init_val.title,
            work: init_val.work,
            start: init_val.start,
            end: init_val.end
        },
    });
    const { register, handleSubmit, formState: {errors} } = form;
    const [ errorMessage, setErrorMessage ] = useState("");
    const navi = useNavigate();

    const {mutate, isPending} = useMutation( {
        mutationFn: (data) => send_data(data),
        onSuccess: () => navi('/'),
        onError: (error) => setErrorMessage(error),
    });

    const onCancelClick = () => { navi('/'); };

    return (
        <>
            <Container>
                <FormProvider {...form}>
                    <VStack as="form" onSubmit={handleSubmit((data)=>mutate(data))}>
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
                        <HStack>
                            <Button 
                                type="submit" 
                                w="30%" 
                                ml="auto" 
                                mr="5%"
                                loading={isPending}
                                loadingTect="送信中"
                            >
                                送信
                            </Button>
                            <Button w="30%" 
                                mr="auto"
                                disabled={isPending}
                                onClick={onCancelClick}
                            >
                                キャンセル
                            </Button>
                        </HStack>
                        <Text> {errorMessage} </Text>
                    </VStack>
                </FormProvider>
            </Container>
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

