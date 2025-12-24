declare module '*.wasm' {
    const content: string;
    export default content;
}

// Stub types for Android/iOS native components (not using React Native)
// These files are placeholders and won't be used in web builds
declare module 'android-native' {
    export interface StyleProp<T> {}
    export interface TurboModule {}
    export const SafeAreaView: any
    export const ScrollView: any
    export const Text: any
    export const TouchableOpacity: any
    export const View: any
    export const TextInput: any
    export const TurboModuleRegistry: { get: <T>(name: string) => T | null }
    export const NativeModules: Record<string, any>
}

declare module 'ios-native' {
    export interface StyleProp<T> {}
    export const SafeAreaView: any
    export const ScrollView: any
    export const Text: any
    export const TouchableOpacity: any
    export const View: any
    export const TextInput: any
}
