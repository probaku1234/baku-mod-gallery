"use client";

import {
  Button,
  Label,
  Modal,
  TextInput,
  Select,
  Spinner,
} from "flowbite-react";
import { createOrUpdatePost } from "../actions";
import QuillTextEditor from "./QuillTextEditor";
import { RxPlus } from "react-icons/rx";
import { useState, useEffect } from "react";
import { Tooltip } from "flowbite-react";
import Image from "next/image";
import { useFormState, useFormStatus } from "react-dom";
import { useRouter } from "next/navigation";

interface Props {
  isOpen: boolean;
  onClose: () => void;
  post?: IPost;
}

const PostModal = (props: Props) => {
  const [imageUrls, setImageUrls] = useState(
    props.post ? props.post.images_url : []
  );
  const [content, setContent] = useState<string>(
    props.post ? props.post.content : ""
  );
  const { pending } = useFormStatus();
  const initialFormState = {
    result: "",
    message: "",
  };
  const [formState, formAction] = useFormState(
    createOrUpdatePost,
    initialFormState
  );
  const router = useRouter();

  const imageUrlTooltipContent = () => {
    return (
      <>
        <Image
          src="https://ac-p1.namu.la/20240116sac/b593bf5655f3533ba51f7141f107198153b7769b75ef727a1a735e74bf73a02b.png?expires=1706117467&key=Dyjt8aawJkorrkxIE8Pdvg&type=orig"
          width={800}
          height={800}
          className="w-16 md:w-32 max-w-full max-h-full"
          alt="Picture of the author"
        />
      </>
    );
  };
  // const createOrUpdatePostWithId = createOrUpdatePost.bind(
  //   null,
  //   props.post ? props.post._id.$oid : "",
  //   content
  // );

  const handleClickPlusIcon = () => {
    setImageUrls([...imageUrls, ""]);
  };

  const onContentChange = (content: string) => {
    setContent(content);
  };

  useEffect(() => {
    if (formState.result) {
      if (formState.result === "success") {
        props.onClose();
      }
      alert(formState.message);
    }
  }, [formState.message, formState.result, props]);
  return (
    <>
      <Modal show={props.isOpen} onClose={props.onClose}>
        {pending && (
          <div className="absolute bg-white bg-opacity-60 z-10 h-full w-full flex items-center justify-center">
            <Spinner size="xl" />
          </div>
        )}
        <Modal.Header>
          Post {props.post ? `Edit (ID: ${props.post._id.$oid})` : "Create"}
        </Modal.Header>
        <Modal.Body>
          <form
            className="flex flex-col gap-4"
            action={formAction}
            method="POST"
          >
            <TextInput
              type="hidden"
              name="postId"
              value={props.post ? props.post._id.$oid : ""}
            />
            <TextInput type="hidden" name="content" value={content} />
            <div>
              <div className="mb-2 block">
                <Label htmlFor="title">Title</Label>
              </div>
              <TextInput
                id="title"
                type="text"
                name="title"
                defaultValue={props.post ? props.post.title : undefined}
                placeholder={props.post ? undefined : "title"}
                required
              />
            </div>
            <div>
              <div className="mb-2 flex content-between items-center">
                <Label htmlFor="url">Image Urls</Label>
                <RxPlus
                  className="align-middle justify-end cursor-pointer"
                  onClick={() => handleClickPlusIcon()}
                />
              </div>
              <div className="gap-4">
                {imageUrls.map((url, index) => (
                  <div key={`url${index}`} className="w-full gap-4">
                    <Tooltip
                      content={imageUrlTooltipContent()}
                      placement="auto"
                    >
                      <TextInput
                        id={`url${index}`}
                        className="w-full"
                        type="url"
                        name={`imageUrl${index}`}
                        defaultValue={
                          props.post ? props.post.images_url[index] : undefined
                        }
                        placeholder={props.post ? undefined : "URL"}
                        required
                      />
                    </Tooltip>
                  </div>
                ))}
              </div>
            </div>
            <div>
              <div className="mb-2 block">
                <Label htmlFor="download_url">Download Url</Label>
              </div>
              <TextInput
                id="download_url"
                name="fileUrl"
                type="url"
                defaultValue={props.post ? props.post.file_url : undefined}
                placeholder={props.post ? undefined : "URL"}
                required
              />
            </div>
            <div>
              <div className="mb-2 block">
                <Label htmlFor="mod_type">Mod Type</Label>
              </div>
              <Select
                id="mod_type"
                name="modType"
                defaultValue={props.post ? props.post.mod_type : "Outfit"}
                required
              >
                <option>Outfit</option>
                <option>Preset</option>
                <option>Follower</option>
              </Select>
            </div>
            <div className="h-2/4">
              <QuillTextEditor
                onChange={onContentChange}
                defaultValue={props.post ? props.post.content : ""}
              />
            </div>
            <Button type="submit">Submit</Button>
          </form>
        </Modal.Body>
        <Modal.Footer>
          <Button color="red" onClick={props.onClose}>
            Cancel
          </Button>
        </Modal.Footer>
      </Modal>
    </>
  );
};

export default PostModal;
