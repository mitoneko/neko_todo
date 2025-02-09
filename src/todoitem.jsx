// todoリストの各アイテム
import {useNavigate} from "react-router-dom";
import {useMutation, useQueryClient} from "@tanstack/react-query";
import {invoke} from "@tauri-apps/api/core";
import { SimpleGrid, GridItem, IconButton, Text, HStack, Container } from "@yamada-ui/react";
import { GrWorkshop } from "react-icons/gr";
import { BsAlarm } from "react-icons/bs";
import { BsEmojiGrin } from "react-icons/bs";
import { BiPencil } from "react-icons/bi";
import { FaRegCopy } from "react-icons/fa6";

export default function TodoItem({item}) {
    const navi = useNavigate();
    const queyrClient = useQueryClient();
    const {mutate} = useMutation({
        mutationFn: () => {
            return invoke("update_done", {id: item.id, done: !item.done})
        },
        onSuccess: () => {
            queyrClient.invalidateQueries({ queryKey: ["todo_list"]});
        }
    });

    const onEditClick = () => {
        navi("/edittodo/"+item.id);
    }

    const onPasteClick = () => {
        navi("/pastetodo/"+item.id);
    }

    const onDoneClick = () => {
        console.log(item.id + " : " + item.title);
        mutate();
    }

    // 日付の表示内容生成
    let end_date = new Date(item.end_date); 
    if (item.end_date === "9999-12-31") {
        end_date = null;
    } 
    const start_date = new Date(item.start_date);
    const update_date = new Date(item.update_date);

    // 完了ボタンのアイコン選択
    let done_icon;
    if (item.done) {
        done_icon = <BsEmojiGrin/>;
    } else if (!!end_date && geDate(new Date(), end_date)) {
        done_icon = <BsAlarm/>;
    } else {
        done_icon = <GrWorkshop/>
    }

    // 上部バーの背景色
    const oneday = 24 * 60 * 60 * 1000;
    const today = new Date();
    const delivery = new Date(item.end_date);
    const daysToDelivery = (delivery - today) / oneday;
    let line_color;
    if (daysToDelivery < 0) {
        line_color = "danger";
    } else if (daysToDelivery < 2) {
        line_color = "warning";
    } else {
        line_color = "success";
    }

    return (
        <Container p="1%" gap="0">
            <SimpleGrid w="full" columns={{base: 2, md: 1}} gap="md" bg={line_color}>
                <GridItem> 
                    <HStack>
                        <IconButton size="xs" icon={done_icon} onClick={onDoneClick} 
                            bg={line_color}/>  
                        <IconButton size="xs" icon={<BiPencil/>} onClick={onEditClick} 
                            bg={line_color}/>
                        <IconButton size="xs" icon={<FaRegCopy/>} onClick={onPasteClick} 
                            bg={line_color}/>
                    </HStack>
                </GridItem>
                
                <GridItem>
                    <Text fontSize="xs" align="right">
                        {update_date?.toLocaleDateString()}
                    </Text>
                </GridItem>
            </SimpleGrid>
            <Text align="center" fontSize="lg" as="b">
                {item.title}
            </Text>
            <Text fontSize="sm">
                {item.work}
            </Text>
            <Text fontSize="sm">
                {start_date?.toLocaleDateString()} 〜 {end_date?.toLocaleDateString()}
            </Text>
        </Container>
    );
}

function geDate(val1, val2) {
    const year1 = val1.getFullYear();
    const month1 = val1.getMonth();
    const day1 = val1.getDate();
    const year2 = val2.getFullYear();
    const month2 = val2.getMonth();
    const day2 = val2.getDate();

    if (year1 === year2) {
        if (month1 === month2) {
            return day1 >= day2;
        } else {
            return month1 > month2;
        }
    } else {
        return year1 > year2;
    }
}

