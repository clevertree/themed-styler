import type { TurboModule } from 'react-native';
export interface Spec extends TurboModule {
    renderCss(usageJson: string, themesJson: string): string;
    getRnStyles(selector: string, classesJson: string, themesJson: string): string;
    getDefaultState(): string;
    getVersion(): string;
}
declare const _default: any;
export default _default;
