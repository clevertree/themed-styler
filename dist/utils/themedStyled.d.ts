import React from 'react';
type StyledProps<T extends React.ComponentType<any>> = Omit<React.ComponentProps<T>, 'style'> & {
    className?: string;
    style?: React.ComponentProps<T>['style'] | Array<React.ComponentProps<T>['style']>;
};
export declare function styled<T extends React.ComponentType<any>>(Component: T, tagName?: string): React.ForwardRefExoticComponent<React.PropsWithoutRef<StyledProps<T>> & React.RefAttributes<React.ComponentRef<T>>>;
export {};
