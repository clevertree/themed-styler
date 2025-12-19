import React from 'react';
export declare function useThemedStyles(tag: string, className?: string): {
    style: any;
};
export declare function resolveThemedStyle(tag: string, className?: string): any;
export declare const ThemedElement: React.ForwardRefExoticComponent<Omit<any, "ref"> & React.RefAttributes<any>>;
export declare const TSDiv: React.ForwardRefExoticComponent<Omit<any, "ref"> & React.RefAttributes<any>>;
