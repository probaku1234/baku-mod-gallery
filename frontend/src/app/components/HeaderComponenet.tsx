import Image from "next/image";

export const HeaderComponent = () => {
  return (
    <div className="flex justify-center">
      <Image src="/profile.png" width={120} height={120} alt="profile picture"/>
    </div>
  );
};
