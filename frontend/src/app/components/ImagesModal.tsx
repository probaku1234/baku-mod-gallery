"use client";

import {
  Button,
  Label,
  Modal,
  TextInput,
  Select,
  Spinner,
} from "flowbite-react";
import Image from "next/image";
import { useState, useEffect } from "react";

const images = ["/profile.png", "/empty.png"];

interface Props {
  isOpen: boolean;
  startIndex: number;
  //   onClose: () => void;
  post?: IPost;
}

const ImagesModal = (props: Props) => {
  const [currentIndex, setCurrentIndex] = useState(0);

  const onPrevClick = () => {
    let prevIndex = currentIndex - 1;

    if (prevIndex < 0) {
      prevIndex = images.length - 1;
    }

    setCurrentIndex(prevIndex);
  };

  const onNextClick = () => {
    let nextIndex = currentIndex + 1;

    if (nextIndex == images.length) {
      nextIndex = 0;
    }

    setCurrentIndex(nextIndex);
  };

  useEffect(() => {
    setCurrentIndex(props.startIndex);
  }, [props.startIndex]);

  return (
    <div className="absolute bg-white bg-opacity-60 z-10 h-full w-full flex items-center justify-center">
      {/* <Modal show={props.isOpen} className="w-full"> */}
      {/* <div id="controls-carousel" className="relative w-full"> */}
      {/* <div className="relative h-56 overflow-hidden rounded-lg md:h-96">
          
        </div> */}
      <Image src={images[currentIndex]} width={400} height={400} alt="sex" />
      <Button
        type="button"
        className="absolute top-0 start-0 z-30 flex items-center justify-center h-full px-4 cursor-pointer group focus:outline-none"
        onClick={onPrevClick}
      >
        <span className="inline-flex items-center justify-center w-10 h-10 rounded-full bg-white/30 dark:bg-gray-800/30 group-hover:bg-white/50 dark:group-hover:bg-gray-800/60 group-focus:ring-4 group-focus:ring-white dark:group-focus:ring-gray-800/70 group-focus:outline-none">
          <svg
            className="w-4 h-4 text-white dark:text-gray-800 rtl:rotate-180"
            aria-hidden="true"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 6 10"
          >
            <path
              stroke="currentColor"
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M5 1 1 5l4 4"
            />
          </svg>
          <span className="sr-only">Previous</span>
        </span>
      </Button>
      <div className="absolute top-0 right-0 flex h-full items-center justify-center px-4 focus:outline-none">
        <Button
          // type="button"
          //   className="absolute top-0 end-0 z-30 flex items-center justify-center h-full px-4 cursor-pointer group focus:outline-none"
          onClick={onNextClick}
        >
          <span className="inline-flex items-center justify-center w-10 h-10 rounded-full bg-white/30 dark:bg-gray-800/30 group-hover:bg-white/50 dark:group-hover:bg-gray-800/60 group-focus:ring-4 group-focus:ring-white dark:group-focus:ring-gray-800/70 group-focus:outline-none">
            <svg
              className="w-4 h-4 text-white dark:text-gray-800 rtl:rotate-180"
              aria-hidden="true"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 6 10"
            >
              <path
                stroke="currentColor"
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth="2"
                d="m1 9 4-4-4-4"
              />
            </svg>
            <span className="sr-only">Next</span>
          </span>
        </Button>
      </div>

      {/* </div> */}

      {/* </Modal> */}
    </div>
  );
};

export default ImagesModal;
