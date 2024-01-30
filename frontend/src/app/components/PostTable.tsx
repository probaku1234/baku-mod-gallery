"use client";

import { Button, Tooltip, Table, Label, TextInput } from "flowbite-react";
import Link from "next/link";
import Image from "next/image";
import { useState } from "react";
import PostModal from "../components/PostModal";
import { Toast } from "flowbite-react";
import { HiCheck, HiExclamation, HiX } from "react-icons/hi";
import debounce from "debounce";
import { deleteAllPosts, deletePost } from "@/app/actions";
import { useFormState } from "react-dom";

interface Props {
  posts: IPost[];
}

const initialFormState = {
  result: "",
  message: "",
};

const PostTable = (props: Props) => {
  const [isOpen, setOpen] = useState(false);
  const [targetPost, setTargetPost] = useState<IPost | undefined>(undefined);
  const [isToastHidden, setToastHidden] = useState(true);
  const [searchKeyword, setSearchKeyword] = useState("");
  const [filteredPosts, setFilteredPosts] = useState<IPost[]>(
    props.posts.slice()
  );
  const [deleteAllFromState, deleteAllFormAction] = useFormState(
    deleteAllPosts,
    initialFormState
  );
  const [deleteFromState, deleteFormAction] = useFormState(
    deletePost,
    initialFormState
  );

  const onClose = () => {
    setOpen(false);
  };

  const openCreatePostModal = () => {
    setTargetPost(undefined);
    setOpen(true);
  };

  const openEditPostModal = (post: IPost) => {
    setTargetPost(post);
    setOpen(true);
  };

  const filterPosts = (keyword: string) => {
    console.log(keyword);
    const result = props.posts.filter((post) => post.title.includes(keyword));
    setFilteredPosts([...result]);
  };
  const onFilterPosts = debounce(filterPosts, 500);

  return (
    <>
      {/* {!isToastHidden && (
        <Toast className="absolute top-0 right-0">
          <div className="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-green-100 text-green-500 dark:bg-green-800 dark:text-green-200">
            <HiCheck className="h-5 w-5" />
          </div>
          <div className="ml-3 text-sm font-normal">
            Item moved successfully.
          </div>
          <Toast.Toggle />
        </Toast>
      )} */}
      {/* {!pending && (
        <div className="text-center">
          <Spinner size="xl" />
        </div>
      )} */}

      <div className="pb-4 bg-white dark:bg-gray-900">
        <Label htmlFor="table-search" className="sr-only">
          Search
        </Label>
        <div className="relative mt-1">
          <div className="absolute inset-y-0 rtl:inset-r-0 start-0 flex items-center ps-3 pointer-events-none">
            <svg
              className="w-4 h-4 text-gray-500 dark:text-gray-400"
              aria-hidden="true"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 20 20"
            >
              <path
                stroke="currentColor"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="m19 19-4-4m0-7A7 7 0 1 1 1 8a7 7 0 0 1 14 0Z"
              />
            </svg>
          </div>
          <TextInput
            type="text"
            id="table-search"
            className="block pt-2 ps-10 text-sm text-gray-900 border border-gray-300 rounded-lg w-80 bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500"
            placeholder="Search for items"
            onChange={(e) => {
              setSearchKeyword(e.target.value);
              onFilterPosts(e.target.value);
            }}
            value={searchKeyword}
          />
        </div>
      </div>
      <Table className="w-full text-sm text-left rtl:text-right text-gray-500 dark:text-gray-400">
        <Table.Head className="text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400">
          <Table.HeadCell scope="col" className="px-16 py-3">
            <span className="sr-only">Image</span>
          </Table.HeadCell>
          <Table.HeadCell scope="col" className="px-6 py-3">
            Title
          </Table.HeadCell>
          <Table.HeadCell scope="col" className="px-6 py-3">
            Download
          </Table.HeadCell>
          <Table.HeadCell scope="col" className="px-6 py-3">
            <span className="sr-only">Edit</span>
          </Table.HeadCell>
        </Table.Head>
        <Table.Body>
          {filteredPosts.map((post) => (
            <Table.Row
              key={post._id.$oid}
              className="bg-white border-b dark:bg-gray-800 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600"
            >
              <Table.Cell className="p-4">
                <Tooltip content="Hello My Tooltip" placement="auto">
                  <Image
                    src="https://ac-p1.namu.la/20240116sac/b593bf5655f3533ba51f7141f107198153b7769b75ef727a1a735e74bf73a02b.png?expires=1706117467&key=Dyjt8aawJkorrkxIE8Pdvg&type=orig"
                    width={400}
                    height={400}
                    className="w-16 md:w-32 max-w-full max-h-full"
                    alt="Picture of the author"
                  />
                </Tooltip>
              </Table.Cell>
              <Table.Cell
                scope="row"
                className="px-6 py-4 font-medium text-gray-900 whitespace-nowrap dark:text-white"
              >
                <Link href={`/posts/detail/${post._id.$oid}`}>
                  {post.title}
                </Link>
              </Table.Cell>
              <Table.Cell className="px-6 py-4">0</Table.Cell>
              <Table.Cell className="px-6 py-4 text-right">
                <div className="flex gap-4 content-center">
                  <Button color="blue" onClick={() => openEditPostModal(post)}>
                    Edit
                  </Button>
                  <form
                    action={deleteFormAction}
                    onSubmit={() => {
                      if (
                        !confirm(
                          "This will delete this posts! Are you sure you ok with this?"
                        )
                      ) {
                        return false;
                      }
                    }}
                  >
                    <input
                      name="postId"
                      className="hidden"
                      value={post._id.$oid}
                    />
                    <Button color="failure" type="submit">
                      Delete
                    </Button>
                  </form>
                </div>
              </Table.Cell>
            </Table.Row>
          ))}
        </Table.Body>
      </Table>
      <div className="flex gap-2">
        <Button onClick={() => openCreatePostModal()}>Create New Post</Button>
        <form
          action={deleteAllFormAction}
          onSubmit={() => {
            if (
              !confirm(
                "This will delete all posts! Are you sure you ok with this?"
              )
            ) {
              return false;
            }
          }}
        >
          <Button color="failure" type="submit">
            Delete All Posts
          </Button>
        </form>
      </div>

      <PostModal isOpen={isOpen} onClose={onClose} post={targetPost} />
    </>
  );
};

export default PostTable;
