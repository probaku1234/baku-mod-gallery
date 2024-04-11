"use client"

import styles from "./page.module.css";
import { Suspense } from "react";
import { PostsFeed } from "./components/PostsFeed";
import Loading from "./loading";
import {
  Navbar,
  NavbarBrand,
  NavbarCollapse,
  NavbarLink,
  NavbarToggle,
  Label,
  TextInput,
} from "flowbite-react";
import Link from "next/link";
import Image from "next/image";

export default function Home() {
  return (
    <main className={styles.main}>
      <Navbar fluid rounded className="w-full">
        {/* <NavbarBrand as={Link} href="https://flowbite-react.com">
          <Image
            src="/vercel.svg"
            width={400}
            height={400}
            className="mr-3 h-6 sm:h-9"
            alt="Flowbite React Logo"
          />
          <span className="self-center whitespace-nowrap text-xl font-semibold dark:text-white">
            Flowbite React
          </span>
        </NavbarBrand> */}
        <NavbarCollapse>
          {/* <NavbarLink href="#" active >
            Home
          </NavbarLink>
          <NavbarLink as={Link} href="#">
            About
          </NavbarLink>
          <NavbarLink href="#">Services</NavbarLink>
          <NavbarLink href="#">Pricing</NavbarLink>
          <NavbarLink href="#">Contact</NavbarLink> */}
        </NavbarCollapse>
        <NavbarToggle />
        <NavbarCollapse>
          <div className="relative overflow-x-auto shadow-md sm:rounded-lg">
            <div className="bg-white dark:bg-gray-900">
              <Label htmlFor="table-search" className="sr-only">
                Search
              </Label>
              <div className="relative">
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
                  className="block ps-10 text-sm text-gray-900 rounded-lg w-80 bg-gray-50 dark:bg-gray-700 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 border-transparent"
                  placeholder="Search for items"
                  // onChange={(e) => {
                  //   setSearchKeyword(e.target.value);
                  //   onFilterPosts(e.target.value);
                  // }}
                  // value={searchKeyword}
                />
              </div>
            </div>
          </div>
        </NavbarCollapse>
      </Navbar>
      <section>
        <Suspense fallback={<Loading />}>
          <PostsFeed />
        </Suspense>
      </section>
    </main>
  );
}
