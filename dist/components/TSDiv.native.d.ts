import React from 'react';
import { StyleProp } from 'react-native';
export declare function useThemedStyles(tag: string, className?: string): {
    style: any;
};
type ThemedElementProps = {
    component: React.ComponentType<any>;
    tag?: string;
    className?: string;
    style?: StyleProp<any>;
    children?: React.ReactNode;
} & Record<string, any>;
export declare const ThemedElement: React.ForwardRefExoticComponent<Omit<ThemedElementProps, "ref"> & React.RefAttributes<any>>;
export declare function resolveThemedStyle(tag: string, className?: string): any;
type TSDivProps = {
    component?: React.ComponentType<any>;
    tag?: string;
    className?: string;
    style?: StyleProp<any>;
    children?: React.ReactNode;
} & Record<string, any>;
export declare const TSDiv: React.ForwardRefExoticComponent<Omit<TSDivProps, "ref"> & React.RefAttributes<any>>;
export {};
