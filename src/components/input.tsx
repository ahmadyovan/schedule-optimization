import React from 'react';

type InputProps = {
  type?: string;
  id?: string;
  name?: string;
  accept?: string;
  placeholder?: string;
  label?: string;
  value?: string;
  min?: number;
  max?: number;
  onChange?: (e: React.ChangeEvent<HTMLInputElement>) => void;
  className?: string;
};

const Input: React.FC<InputProps> = ({
  type = 'text',
  id,
  name,
  accept,
  placeholder,
  label,
  value,
  onChange,
  className,
}) => {
  return (
    <div className="flex flex-col gap-1">
      {label && <label htmlFor={id}>{label}</label>}
      <input
        type={type}
        id={id}
        name={name}
        accept={accept}
        placeholder={placeholder}
        value={value}
        onChange={onChange}
        className={`border rounded px-3 py-2 ${className}`}
      />
    </div>
  );
};

export default Input;
