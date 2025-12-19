import { SafeAreaView as RNSafeAreaView, ScrollView as RNScrollView, Text as RNText, View as RNView, TextInput as RNTextInput } from 'react-native';
export declare const SafeAreaView: import("react").ForwardRefExoticComponent<Omit<import("react-native").ViewProps, "style"> & {
    className?: string;
    style?: import("react-native").StyleProp<import("react-native").ViewStyle> | import("react-native").StyleProp<import("react-native").ViewStyle>[];
} & import("react").RefAttributes<RNSafeAreaView>>;
export declare const ScrollView: import("react").ForwardRefExoticComponent<Omit<import("react-native").ScrollViewProps, "style"> & {
    className?: string;
    style?: import("react-native").StyleProp<import("react-native").ViewStyle> | import("react-native").StyleProp<import("react-native").ViewStyle>[];
} & import("react").RefAttributes<RNScrollView>>;
export declare const Text: import("react").ForwardRefExoticComponent<Omit<import("react-native").TextProps, "style"> & {
    className?: string;
    style?: import("react-native").StyleProp<import("react-native").TextStyle> | import("react-native").StyleProp<import("react-native").TextStyle>[];
} & import("react").RefAttributes<RNText>>;
export declare const TextInput: import("react").ForwardRefExoticComponent<Omit<import("react-native").TextInputProps, "style"> & {
    className?: string;
    style?: import("react-native").StyleProp<import("react-native").TextStyle> | import("react-native").StyleProp<import("react-native").TextStyle>[];
} & import("react").RefAttributes<RNTextInput>>;
export declare const TouchableOpacity: import("react").ForwardRefExoticComponent<Omit<Omit<import("react-native").TouchableOpacityProps & import("react").RefAttributes<RNView>, "style"> & {
    className?: string;
    style?: import("react-native").StyleProp<import("react-native").ViewStyle> | import("react-native").StyleProp<import("react-native").ViewStyle>[];
}, "ref"> & import("react").RefAttributes<RNView>>;
export declare const View: import("react").ForwardRefExoticComponent<Omit<import("react-native").ViewProps, "style"> & {
    className?: string;
    style?: import("react-native").StyleProp<import("react-native").ViewStyle> | import("react-native").StyleProp<import("react-native").ViewStyle>[];
} & import("react").RefAttributes<RNView>>;
